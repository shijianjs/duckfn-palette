---
title: Conversion and clamping
sidebar_position: 2
description: Move a color between any two of the 44 spaces, with or without clamping, check whether it is inside its space's range, and compare two colors as rendered.
---

# Conversion and clamping

Conversion always keeps the space tag and alpha consistent: after `plt_convert(c, 'lab')` the value
says `lab`, and `plt_convert(..., 'srgb')` brings it back. Palette performs the math; this extension
only chooses the route.

## `plt_convert(color, space)`

**Signature**: `plt_convert(color plt_color, space VARCHAR) -> plt_color`

Converts to the target space **without gamut clamping**. Values outside the target's range are
returned as they are, which is useful for inspecting intermediate results and for conversions that
are supposed to be reversible.

```sql {"type":"duckfn","show":"table","colors":"srgb"}
SELECT plt_to_hex(plt_srgb(1, 0.5, 0)) AS srgb,
       plt_components(plt_convert(plt_srgb(1, 0.5, 0), 'oklch')) AS as_oklch,
       plt_components(plt_convert(plt_srgb(1, 0.5, 0), 'xyz')) AS as_xyz;
```

Alpha is copied, not converted:

```sql {"type":"duckfn","show":"table"}
SELECT plt_convert(plt_with_alpha(plt_srgb(1, 0, 0), 0.25), 'lab') AS converted,
       plt_alpha(plt_convert(plt_with_alpha(plt_srgb(1, 0, 0), 0.25), 'lab')) AS alpha;
```

## `plt_convert_clamped(color, space)`

**Signature**: `plt_convert_clamped(color plt_color, space VARCHAR) -> plt_color`

Converts, then clamps the result to the **target** space's standard range. This is what you want
before rendering: a color outside sRGB comes back as the nearest in-gamut sRGB color instead of
`1.5` or `-0.3` components.

```sql {"type":"duckfn","show":"table"}
SELECT plt_components(plt_convert(plt_lab(50, 200, 200), 'srgb')) AS unclamped,
       plt_components(plt_convert_clamped(plt_lab(50, 200, 200), 'srgb')) AS clamped;
```

## `plt_clamp(color)`

**Signature**: `plt_clamp(color plt_color) -> plt_color`

Clamps in the color's **own** space, without converting. Lab's a\* and b\* stop at ±127, RGB
components at 0..1, but a hue is not forced into 0..360 by this function.

```sql {"type":"duckfn","show":"table"}
SELECT plt_components(plt_lab(50, 200, 200)) AS out_of_range,
       plt_components(plt_clamp(plt_lab(50, 200, 200))) AS clamped,
       plt_components(plt_srgb(-0.2, 1.4, 0.5)) AS out_of_gamut,
       plt_components(plt_clamp(plt_srgb(-0.2, 1.4, 0.5))) AS in_gamut;
```

## `plt_is_within_bounds(color)`

**Signature**: `plt_is_within_bounds(color plt_color) -> BOOLEAN`

Whether `plt_clamp` would change anything. Combine it with a `WHERE` clause to find colors that a
renderer would have to clip.

```sql {"type":"duckfn","show":"table"}
SELECT plt_is_within_bounds(plt_lab(50, 200, 200)) AS lab_ok,
       plt_is_within_bounds(plt_clamp(plt_lab(50, 200, 200))) AS clamped_ok,
       plt_is_within_bounds(plt_srgb(1, 0, 0)) AS srgb_ok;
```

```sql {"type":"duckfn","show":"table"}
SELECT c, plt_is_within_bounds(c) AS in_gamut
FROM (VALUES
        (plt_srgb(0.2, 0.4, 0.6)),
        (plt_srgb(-0.1, 0.5, 0.5)),
        (plt_lch(50, 120, 40))
     ) t(c);
```

## Which route does a conversion take?

All conversions go through XYZ (D65), the hub of palette's conversion graph. Encoded RGB spaces are
decoded first; D50 spaces (ProPhoto RGB, `xyz_d50`) are adapted with Bradford; CAM16 uses palette's
default viewing conditions. You do not choose the route, but `plt_convert` and
`plt_convert_clamped` produce the same numbers on it, and one round trip through any invertible
space returns the original color to floating-point precision:

```sql {"type":"duckfn","show":"table","colors":"round_trip"}
SELECT plt_to_hex(plt_convert(plt_convert(plt_srgb(0.2, 0.4, 0.6), 'hsluv'), 'srgb')) AS round_trip;
```

The one-dimensional `luma*` spaces are not invertible by design, and `srgb_gamma` is palette's
deprecated gamma 2.2 encoding — use `plt_srgb` unless you specifically need it.

## `plt_equals(a, b)`

**Signature**: `plt_equals(a plt_color, b plt_color) -> BOOLEAN`

Whether two colors are the same once rendered: both are converted to sRGB, quantized to 8 bits per
channel, and alpha is rounded to two decimals. This is tinycolor's `equals`, and it is usually what
you mean when comparing colors from different spaces — `plt_srgb(1, 0, 0)` and `plt_hsl(0, 1, 0.5)`
are equal. Two colors that a perceptually uniform space might separate by a hair still count as
equal here:

```sql {"type":"duckfn","show":"table"}
SELECT plt_equals(plt_srgb(1, 0, 0), plt_hsl(0, 1, 0.5)) AS across_spaces,
       plt_equals(plt_with_alpha(plt_srgb(1, 0, 0), 0.5), plt_with_alpha(plt_srgb(1, 0, 0), 0.6)) AS alpha_differs,
       plt_equals(plt_srgb(0.5, 0.5, 0.5), plt_srgb(0.501, 0.5, 0.5)) AS within_a_byte;
```

Comparing the STRUCTs directly with `=` does not do this: it also compares the `space` tag and the
full floating-point components, so two `srgb` values that render identically but differ in the last
bit are `!=` there.

## Errors

An unknown space name fails the statement; the message names the function:

```sql {"type":"duckfn","expect":"error"}
SELECT plt_convert(plt_srgb(1, 0, 0), 'nope');
```

```sql {"type":"duckfn","expect":"error"}
SELECT plt_convert_clamped(plt_srgb(1, 0, 0), 'not-a-space');
```
