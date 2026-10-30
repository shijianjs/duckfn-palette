---
title: White points, adaptation and encoding
sidebar_position: 1
description: Named white points, chromatic adaptation matrices, transfer functions, RGB primaries, RGB-to-XYZ matrices, and the general 3x3 matrix helpers.
---

# White points, adaptation and encoding

`plt_convert` already handles all of this internally — it adapts D50 spaces to D65 with Bradford and
decodes transfer functions on the way. The functions on this page expose the same pieces so you can
inspect them, reproduce a conversion by hand, or combine them into something `plt_convert` does not
do.

Nothing here changes a color's space. The one function that returns a color,
`plt_chromatic_adapt`, keeps the `xyz` tag and changes the numbers.

## White points

### `plt_white_points()`

**Signature**: `plt_white_points() -> VARCHAR[]`

Every white point name: the CIE illuminants `a`, `b`, `c`, `d50`, `d55`, `d65`, `d75`, `e`, `f2`,
`f7`, `f11`, their 10° observer variants `d50_degree10`, `d55_degree10`, `d65_degree10`,
`d75_degree10`, and `dci_p3` (the theatre white).

```sql {"type":"duckfn","show":"table"}
SELECT plt_white_points() AS white_points;
```

### `plt_white_point_xyz(name)`

**Signature**: `plt_white_point_xyz(name VARCHAR) -> DOUBLE[]`

The tristimulus values `[X, Y, Z]`, normalized so `Y = 1`. This is the `[0.95047, 1, 1.08883]` you
recognize from D65:

```sql {"type":"duckfn","show":"table"}
SELECT plt_white_point_xyz('d65') AS d65,
       plt_white_point_xyz('d50') AS d50,
       plt_white_point_xyz('a') AS illuminant_a;
```

### `plt_white_point_xy(name)`

**Signature**: `plt_white_point_xy(name VARCHAR) -> DOUBLE[]`

The chromaticity coordinates `[x, y]` instead of XYZ — the position on the CIE 1931 diagram:

```sql {"type":"duckfn","show":"table"}
SELECT plt_white_point_xy('d65') AS d65,
       plt_white_point_xy('d50') AS d50;
```

Names are normalized (`D65`, `d-65`, `d 65` all work), and a few aliases are accepted:
`illuminant_a` … `illuminant_e`, and `d65_10` / `d65_10deg` / `d65_degree_10` for the 10° variant.

```sql {"type":"duckfn","expect":"error"}
SELECT plt_white_point_xyz('d42');
```

## Chromatic adaptation

### `plt_adaptation_methods()`

**Signature**: `plt_adaptation_methods() -> VARCHAR[]`

The three methods: `bradford` (the usual default, and what `plt_convert` uses), `von_kries`, and
`unit_matrix` (a plain XYZ scaling, useful as a baseline). Aliases such as `vonkries`, `hpe`,
`hunt_pointer_estevez`, `identity`, `none` and `xyz_scaling` are accepted.

```sql {"type":"duckfn","show":"table"}
SELECT plt_adaptation_methods() AS methods;
```

### `plt_adaptation_matrix(source, target, method)`

**Signature**: `plt_adaptation_matrix(source VARCHAR, target VARCHAR, method VARCHAR) -> DOUBLE[]`

The 3×3 matrix that takes XYZ values under `source` to XYZ values under `target`, row-major:

```sql {"type":"duckfn","show":"table"}
SELECT plt_adaptation_matrix('d65', 'd50', 'bradford') AS d65_to_d50,
       plt_adaptation_matrix('a', 'c', 'bradford') AS a_to_c;
```

The matrix is built from three pieces, all exposed on their own:
`plt_lms_matrix(method)` → gain → `plt_lms_to_xyz_matrix(method)`. That makes it possible to build a
custom chain:

```sql {"type":"duckfn","show":"table"}
SELECT plt_lms_matrix('bradford') AS xyz_to_lms,
       plt_diagonal_adaptation_matrix('d65', 'd50', 'bradford') AS gains,
       plt_lms_to_xyz_matrix('bradford') AS lms_to_xyz;
```

### `plt_lms_matrix(method)`

**Signature**: `plt_lms_matrix(method VARCHAR) -> DOUBLE[]`

The method's XYZ→LMS matrix (Bradford, von Kries / Hunt-Pointer-Estevez, or the identity-based unit
matrix).

### `plt_lms_to_xyz_matrix(method)`

**Signature**: `plt_lms_to_xyz_matrix(method VARCHAR) -> DOUBLE[]`

The method's LMS→XYZ matrix, the inverse of `plt_lms_matrix`.

### `plt_diagonal_adaptation_matrix(source, target, method)`

**Signature**: `plt_diagonal_adaptation_matrix(source VARCHAR, target VARCHAR, method VARCHAR) -> DOUBLE[]`

The diagonal LMS gain matrix between two white points. It is only meaningful **between** the two LMS
matrices:

```sql {"type":"duckfn","show":"table"}
SELECT plt_diagonal_adaptation_matrix('d65', 'd50', 'bradford') AS gains;
```

### `plt_chromatic_adapt(color, target, method)` and `plt_chromatic_adapt(color, source, target, method)`

**Signature**: `plt_chromatic_adapt(color plt_color, target VARCHAR, method VARCHAR) -> plt_color`
**Signature**: `plt_chromatic_adapt(color plt_color, source VARCHAR, target VARCHAR, method VARCHAR) -> plt_color`

Applies the adaptation matrix to a color. The color **must be in `xyz`**, because its components are
the tristimulus values the matrix operates on. The three-argument form assumes the components are
under D65; the four-argument form takes the source white point explicitly.

The returned value is tagged `xyz` and carries values under the target white point. The white point
itself is not part of the value, so keep track of it yourself — `plt_xyz` means D65, while a
`d50`-adapted result is the same tag with different numbers.

```sql {"type":"duckfn","show":"table"}
SELECT plt_chromatic_adapt(plt_xyz(0.5, 0.4, 0.3), 'd50', 'bradford') AS adapted,
       plt_chromatic_adapt(plt_xyz(0.5, 0.4, 0.3), 'd65', 'd50', 'bradford') AS explicit_source;
```

```sql {"type":"duckfn","show":"table"}
SELECT plt_components(c) AS original,
       plt_components(plt_chromatic_adapt(c, 'a', 'bradford')) AS under_a,
       plt_components(c) AS still_original
FROM (SELECT plt_xyz(0.5, 0.4, 0.3) AS c);
```

Applying the adaptation by hand with the matrix and `plt_apply_matrix` gives the same numbers:

```sql {"type":"duckfn","show":"table"}
SELECT plt_components(plt_chromatic_adapt(c, 'd50', 'bradford')) AS adapted,
       plt_components(plt_apply_matrix(plt_adaptation_matrix('d65', 'd50', 'bradford'), c)) AS by_hand
FROM (SELECT plt_xyz(0.5, 0.4, 0.3) AS c);
```

A color in another space is an error, not an implicit conversion:

```sql {"type":"duckfn","expect":"error"}
SELECT plt_chromatic_adapt(plt_srgb(1, 0, 0), 'd50', 'bradford');
```

```sql {"type":"duckfn","expect":"error"}
SELECT plt_adaptation_matrix('d65', 'd50', 'cat02');
```

## 3x3 matrices

A matrix is always a `LIST(DOUBLE)` of exactly nine values, **row-major**:

```text
[m11, m12, m13, m21, m22, m23, m31, m32, m33]
```

### `plt_matrix_identity()`

**Signature**: `plt_matrix_identity() -> DOUBLE[]`

### `plt_matrix_scale(s1, s2, s3)`

**Signature**: `plt_matrix_scale(s1 DOUBLE, s2 DOUBLE, s3 DOUBLE) -> DOUBLE[]`

A diagonal matrix with the three entries on the diagonal — it scales each component independently.

```sql {"type":"duckfn","show":"table"}
SELECT plt_matrix_identity() AS identity,
       plt_matrix_scale(2, 0.5, 1) AS scale;
```

### `plt_matrix_multiply(a, b)`

**Signature**: `plt_matrix_multiply(a DOUBLE[], b DOUBLE[]) -> DOUBLE[]`

The product that **applies `a` first and then `b`** — palette's `Matrix3::then` order. To adapt and
then scale, pass the adaptation matrix first:

```sql {"type":"duckfn","show":"table"}
SELECT plt_matrix_multiply(plt_matrix_identity(), plt_matrix_scale(2, 2, 2)) AS scaled,
       plt_matrix_multiply(plt_matrix_scale(2, 2, 2), plt_matrix_identity()) AS same;
```

### `plt_matrix_inverse(matrix)`

**Signature**: `plt_matrix_inverse(matrix DOUBLE[]) -> DOUBLE[]`

### `plt_matrix_apply(matrix, x, y, z)`

**Signature**: `plt_matrix_apply(matrix DOUBLE[], x DOUBLE, y DOUBLE, z DOUBLE) -> DOUBLE[]`

Applies a matrix to three numbers and returns the three results — no color type involved:

```sql {"type":"duckfn","show":"table"}
SELECT plt_matrix_apply(plt_matrix_scale(2, 3, 4), 1, 1, 1) AS scaled,
       plt_matrix_apply(plt_matrix_inverse(plt_matrix_scale(2, 4, 0.5)), 2, 4, 0.5) AS back_to_one;
```

### `plt_apply_matrix(matrix, color)`

**Signature**: `plt_apply_matrix(matrix DOUBLE[], color plt_color) -> plt_color`

Applies the matrix to a color's three components **in its current space** and keeps the space tag
and alpha. The typical use is `xyz` plus an adaptation or primaries matrix:

```sql {"type":"duckfn","show":"table"}
SELECT plt_components(plt_apply_matrix(plt_matrix_scale(0.5, 0.5, 0.5),
                                       plt_xyz(0.4, 0.3, 0.2))) AS halved,
       plt_space(plt_apply_matrix(plt_matrix_scale(0.5, 0.5, 0.5), plt_oklch(0.5, 0.1, 30))) AS still_oklch;
```

Errors: a list that is not nine numbers long, and a matrix that cannot be inverted:

```sql {"type":"duckfn","expect":"error"}
SELECT plt_matrix_apply([1, 0, 0, 0, 1, 0], 1, 2, 3);
```

## Encoding: transfer functions, primaries and matrices

### `plt_encoding_spaces()`

**Signature**: `plt_encoding_spaces() -> VARCHAR[]`

The 15 RGB space names plus the three luma names — everything with a transfer function.

```sql {"type":"duckfn","show":"table"}
SELECT plt_encoding_spaces() AS spaces;
```

### `plt_transfer_function(space)`

**Signature**: `plt_transfer_function(space VARCHAR) -> VARCHAR`

Which transfer function a space uses. Encoded and linear variants report different answers:

```sql {"type":"duckfn","show":"table"}
SELECT plt_transfer_function('srgb') AS srgb,
       plt_transfer_function('srgb_linear') AS srgb_linear,
       plt_transfer_function('dci_p3') AS dci_p3,
       plt_transfer_function('prophoto_rgb') AS prophoto,
       plt_transfer_function('rec2020') AS rec2020;
```

### `plt_transfer_functions()`

**Signature**: `plt_transfer_functions() -> VARCHAR[]`

The seven names: `linear`, `gamma_2_2`, `srgb`, `adobe_rgb`, `prophoto_rgb`, `p3_gamma`,
`rec_oetf`.

### `plt_decode(space, value)`

**Signature**: `plt_decode(space VARCHAR, value DOUBLE) -> DOUBLE`

Takes one **encoded** component value to linear light with the space's transfer function. This is
what happens to each RGB component on the way into XYZ:

```sql {"type":"duckfn","show":"table"}
SELECT plt_decode('srgb', 0.5) AS half_encoded,
       plt_decode('srgb', 1.0) AS one,
       plt_decode('srgb_linear', 0.5) AS already_linear;
```

### `plt_encode(space, value)`

**Signature**: `plt_encode(space VARCHAR, value DOUBLE) -> DOUBLE`

The inverse: one linear value back into the space's encoding.

```sql {"type":"duckfn","show":"table"}
SELECT plt_encode('srgb', 0.21404114048223255) AS back_to_half,
       plt_encode('srgb', plt_decode('srgb', 0.25)) AS round_trip;
```

### `plt_white_point_of(space)`

**Signature**: `plt_white_point_of(space VARCHAR) -> VARCHAR`

The white point a space is defined against. This accepts any of the 44 spaces, not just RGB:

```sql {"type":"duckfn","show":"table"}
SELECT plt_white_point_of('srgb') AS srgb,
       plt_white_point_of('prophoto_rgb') AS prophoto,
       plt_white_point_of('xyz_d50') AS xyz_d50,
       plt_white_point_of('dci_p3') AS dci_p3,
       plt_white_point_of('oklch') AS oklch;
```

### `plt_rgb_primaries(space)`

**Signature**: `plt_rgb_primaries(space VARCHAR) -> plt_color[]`

The red, green and blue primaries of an RGB space, each as a `yxy` color (chromaticity plus
luminance), in that order. Luma space names are accepted and share sRGB's primaries:

```sql {"type":"duckfn","show":"table"}
SELECT plt_rgb_primaries('srgb') AS primaries,
       plt_components(plt_rgb_primaries('display_p3')[1]) AS p3_red;
```

### `plt_rgb_to_xyz_matrix(space)`

**Signature**: `plt_rgb_to_xyz_matrix(space VARCHAR) -> DOUBLE[]`

The space's 3×3 RGB→XYZ matrix, row-major. This is the linear part of the conversion:

```sql {"type":"duckfn","show":"table"}
SELECT plt_rgb_to_xyz_matrix('srgb') AS srgb,
       plt_rgb_to_xyz_matrix('rec2020') AS rec2020;
```

### `plt_xyz_to_rgb_matrix(space)`

**Signature**: `plt_xyz_to_rgb_matrix(space VARCHAR) -> DOUBLE[]`

The inverse matrix, XYZ→RGB, computed from the one above:

```sql {"type":"duckfn","show":"table"}
SELECT plt_xyz_to_rgb_matrix('srgb') AS xyz_to_srgb;
```

Reproducing a conversion with the pieces — decode the components, apply the matrix, and compare with
`plt_convert`:

```sql {"type":"duckfn","show":"table"}
SELECT plt_matrix_apply(plt_rgb_to_xyz_matrix('srgb'),
                        plt_decode('srgb', 1), plt_decode('srgb', 0.5), plt_decode('srgb', 0)) AS by_hand,
       plt_components(plt_convert(plt_srgb(1, 0.5, 0), 'xyz')) AS converted;
```

A space without primaries is an error:

```sql {"type":"duckfn","expect":"error"}
SELECT plt_rgb_primaries('lab');
```

```sql {"type":"duckfn","expect":"error"}
SELECT plt_decode('hsl', 0.5);
```
