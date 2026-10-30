---
title: Colors and spaces
sidebar_position: 1
description: The plt_color value, its accessors, alpha handling, and the 44 color-space constructors with their units and ranges.
---

# Colors and spaces

## The color value

Every function takes and returns the same `STRUCT`:

```text
STRUCT(space VARCHAR, c1 DOUBLE, c2 DOUBLE, c3 DOUBLE, alpha DOUBLE)
```

There is no separate `hsl` type or `lab` type — the space name inside the value says how `c1..c3`
must be read. That is what makes `plt_convert` possible and what lets one column hold colors from
different spaces.

```sql {"type":"duckfn","show":"table"}
SELECT plt_srgb(1, 0.5, 0) AS color,
       plt_space(plt_srgb(1, 0.5, 0)) AS space,
       plt_components(plt_srgb(1, 0.5, 0)) AS components,
       plt_alpha(plt_srgb(1, 0.5, 0)) AS alpha;
```

## Universal construction

### `plt_color(space, c1, c2, c3)`

**Signature**: `plt_color(space VARCHAR, c1 DOUBLE, c2 DOUBLE, c3 DOUBLE) -> plt_color`

Builds a color in any supported space from its name. Alpha defaults to 1. The space name accepts
aliases such as `xyz-d65`, `p3`, `prophoto`, `cie-lab`, `ok-lch` or `grey`, in any case.

```sql {"type":"duckfn","show":"table"}
SELECT plt_convert(plt_color('lab', 50, 20, -30), 'srgb') AS from_name,
       plt_color('P3', 1, 0, 0).space AS alias;
```

### `plt_color(space, c1, c2, c3, alpha)`

**Signature**: `plt_color(space VARCHAR, c1 DOUBLE, c2 DOUBLE, c3 DOUBLE, alpha DOUBLE) -> plt_color`

The same, with explicit alpha.

```sql {"type":"duckfn","show":"table"}
SELECT plt_color('oklch', 0.7, 0.1, 120, 0.5) AS translucent;
```

### `plt_from_components(space, components)`

**Signature**: `plt_from_components(space VARCHAR, components DOUBLE[]) -> plt_color`

Builds a color from a list of exactly three components; alpha defaults to 1. This is the inverse of
`plt_components`, so a color survives a round trip through a list.

```sql {"type":"duckfn","show":"table","colors":"rebuilt"}
SELECT plt_to_hex(plt_from_components('srgb', [0.2, 0.4, 0.6])) AS rebuilt;
```

### `plt_from_components_alpha(space, components)`

**Signature**: `plt_from_components_alpha(space VARCHAR, components DOUBLE[]) -> plt_color`

Takes a list of exactly four values: the three components plus alpha.

```sql {"type":"duckfn","show":"table","colors":"translucent"}
SELECT plt_to_hex(plt_from_components_alpha('srgb', [0.2, 0.4, 0.6, 0.5])) AS translucent;
```

## Reading a color

### `plt_space(color)`

**Signature**: `plt_space(color plt_color) -> VARCHAR`

The canonical space name, which is what `plt_color_spaces()` lists. It never returns an alias.

```sql {"type":"duckfn","show":"table"}
SELECT plt_space(plt_color('OkLcH', 0.5, 0.1, 120)) AS canonical;
```

### `plt_components(color)`

**Signature**: `plt_components(color plt_color) -> DOUBLE[]`

All three components in the space's natural order. For the one-dimensional `luma*` spaces the list is
always `[value, 0, 0]`.

```sql {"type":"duckfn","show":"table"}
SELECT plt_components(plt_hsl(120, 0.5, 0.4)) AS hsl,
       plt_components(plt_lch(50, 40, 120)) AS lch,
       plt_components(plt_luma(0.5)) AS luma;
```

### `plt_component(color, index)`

**Signature**: `plt_component(color plt_color, index BIGINT) -> DOUBLE`

One component, 1-based. `index` must be 1, 2 or 3.

```sql {"type":"duckfn","show":"table"}
SELECT plt_component(plt_srgb(0.1, 0.2, 0.3), 1) AS red,
       plt_component(plt_srgb(0.1, 0.2, 0.3), 3) AS blue;
```

### `plt_alpha(color)`

**Signature**: `plt_alpha(color plt_color) -> DOUBLE`

The alpha channel, 0 (transparent) to 1 (opaque).

```sql {"type":"duckfn","show":"table"}
SELECT plt_alpha(plt_srgb(1, 0, 0)) AS opaque,
       plt_alpha(plt_color('srgb', 1, 0, 0, 0.25)) AS translucent;
```

### `plt_with_alpha(color, alpha)`

**Signature**: `plt_with_alpha(color plt_color, alpha DOUBLE) -> plt_color`

Replaces alpha and leaves space and components untouched.

```sql {"type":"duckfn","show":"table"}
SELECT plt_with_alpha(plt_srgb(1, 0, 0), 0.5) AS half_transparent;
```

### `plt_opaque(color)`

**Signature**: `plt_opaque(color plt_color) -> plt_color`

Sets alpha to 1.

```sql {"type":"duckfn","show":"table"}
SELECT plt_alpha(plt_opaque(plt_color('srgb', 1, 0, 0, 0.25))) AS alpha;
```

### `plt_color_spaces()`

**Signature**: `plt_color_spaces() -> VARCHAR[]`

All 44 canonical space names, in documentation order.

```sql {"type":"duckfn","show":"table"}
SELECT list_count(plt_color_spaces()) AS count, plt_color_spaces() AS spaces;
```

## One constructor per space

Every name in `plt_color_spaces()` has a constructor of the same name. All take `DOUBLE` components
in palette's `Color::new` order and set alpha to 1. Components are stored as given (they are not
clipped); `plt_clamp` or `plt_convert_clamped` is how you bring them into range.

### RGB spaces (encoded and linear)

| Constructor | Components | Notes |
| --- | --- | --- |
| `plt_srgb(r, g, b)` | red, green, blue 0..1 | gamma-encoded sRGB, the default output space |
| `plt_srgb_linear(r, g, b)` | red, green, blue 0..1 | linear-light sRGB |
| `plt_srgb_gamma(r, g, b)` | red, green, blue 0..1 | gamma 2.2 encoding (palette's deprecated `GammaSrgb`, still exposed) |
| `plt_adobe_rgb(r, g, b)` | red, green, blue 0..1 | Adobe RGB (1998) |
| `plt_adobe_rgb_linear(r, g, b)` | red, green, blue 0..1 | linear-light Adobe RGB |
| `plt_display_p3(r, g, b)` | red, green, blue 0..1 | Display P3, D65, sRGB transfer function |
| `plt_display_p3_linear(r, g, b)` | red, green, blue 0..1 | linear-light Display P3 |
| `plt_dci_p3(r, g, b)` | red, green, blue 0..1 | DCI-P3, theatre white point, gamma 2.6 |
| `plt_dci_p3_linear(r, g, b)` | red, green, blue 0..1 | linear-light DCI-P3 |
| `plt_prophoto_rgb(r, g, b)` | red, green, blue 0..1 | ProPhoto RGB, D50, gamma 1.8 |
| `plt_prophoto_rgb_linear(r, g, b)` | red, green, blue 0..1 | linear-light ProPhoto RGB |
| `plt_rec709(r, g, b)` | red, green, blue 0..1 | Rec. 709 primaries and transfer function |
| `plt_rec709_linear(r, g, b)` | red, green, blue 0..1 | linear-light Rec. 709 |
| `plt_rec2020(r, g, b)` | red, green, blue 0..1 | Rec. 2020 primaries and transfer function |
| `plt_rec2020_linear(r, g, b)` | red, green, blue 0..1 | linear-light Rec. 2020 |

```sql {"type":"duckfn","show":"table","colors":["srgb","display_p3"]}
SELECT plt_to_hex(plt_srgb(1, 0.5, 0)) AS srgb,
       plt_to_hex(plt_display_p3(1, 0.5, 0)) AS display_p3,
       plt_rgb_to_xyz_matrix('rec2020') AS rec2020_matrix;
```

### Hue-first spaces

These take the hue first. Hue is in degrees and may be negative or above 360.

| Constructor | Components | Ranges |
| --- | --- | --- |
| `plt_hsl(h, s, l)` | hue, saturation, lightness | hue degrees; s and l 0..1 |
| `plt_hsv(h, s, v)` | hue, saturation, value | hue degrees; s and v 0..1 |
| `plt_hwb(h, w, b)` | hue, whiteness, blackness | hue degrees; w and b 0..1 |
| `plt_hsluv(h, s, l)` | hue, saturation, lightness | hue degrees; s and l 0..100 |
| `plt_okhsl(h, s, l)` | hue, saturation, lightness | hue degrees; s and l 0..1 |
| `plt_okhsv(h, s, v)` | hue, saturation, value | hue degrees; s and v 0..1 |
| `plt_okhwb(h, w, b)` | hue, whiteness, blackness | hue degrees; w and b 0..1 |

```sql {"type":"duckfn","show":"table","colors":["hsl","okhsl","hsluv"]}
SELECT plt_to_hex(plt_hsl(120, 0.5, 0.5)) AS hsl,
       plt_to_hex(plt_okhsl(120, 0.5, 0.5)) AS okhsl,
       plt_to_hex(plt_hsluv(120, 50, 50)) AS hsluv;
```

### CIE spaces

| Constructor | Components | Ranges |
| --- | --- | --- |
| `plt_lab(l, a, b)` | L\*, a\*, b\* | L\* 0..100; a\* and b\* roughly -128..127 |
| `plt_lch(l, c, h)` | L\*, chroma, hue | L\* 0..100; c ≥ 0; hue degrees |
| `plt_luv(l, u, v)` | L\*, u\*, v\* | L\* 0..100 |
| `plt_lchuv(l, c, h)` | L\*, chroma, hue | L\* 0..100; c ≥ 0; hue degrees |
| `plt_xyz(x, y, z)` | X, Y, Z | D65 white point; Y = 1 for white |
| `plt_xyz_d50(x, y, z)` | X, Y, Z | D50 white point (conversions adapt with Bradford) |
| `plt_yxy(x, y, luminance)` | chromaticity x, y, luminance | x and y are chromaticity, luminance ≥ 0 |

```sql {"type":"duckfn","show":"table","colors":["lab","lch","xyz"]}
SELECT plt_to_hex(plt_lab(50, 20, -30)) AS lab,
       plt_to_hex(plt_lch(50, 40, 120)) AS lch,
       plt_to_hex(plt_xyz(0.5, 0.4, 0.3)) AS xyz;
```

### Perceptual OK spaces

| Constructor | Components | Ranges |
| --- | --- | --- |
| `plt_oklab(l, a, b)` | lightness, a, b | l 0..1; a and b nominally -1..1 |
| `plt_oklch(l, c, h)` | lightness, chroma, hue | l 0..1; c ≥ 0; hue degrees |

```sql {"type":"duckfn","show":"table","colors":["oklab","oklch"]}
SELECT plt_to_hex(plt_oklab(0.5, 0.1, 0.1)) AS oklab,
       plt_to_hex(plt_oklch(0.5, 0.1, 120)) AS oklch;
```

### CAM16 spaces (default viewing conditions)

These use palette's default conditions: D65, adapting luminance 40, background luminance 0.2,
average surround and automatic discounting. For your own conditions use
[the CAM16 functions](../color-science/cam16.md).

| Constructor | Components |
| --- | --- |
| `plt_cam16_jch(j, c, h)` | lightness J, chroma C, hue |
| `plt_cam16_jmh(j, m, h)` | lightness J, colorfulness M, hue |
| `plt_cam16_jsh(j, s, h)` | lightness J, saturation s, hue |
| `plt_cam16_qch(q, c, h)` | brightness Q, chroma C, hue |
| `plt_cam16_qmh(q, m, h)` | brightness Q, colorfulness M, hue |
| `plt_cam16_qsh(q, s, h)` | brightness Q, saturation s, hue |
| `plt_cam16_ucs_jab(j, a, b)` | CAM16-UCS J′, a′, b′ |
| `plt_cam16_ucs_jmh(j, m, h)` | CAM16-UCS J′, M′, hue |

```sql {"type":"duckfn","show":"table","colors":["jch","ucs_jmh"]}
SELECT plt_to_hex(plt_cam16_jch(50, 40, 120)) AS jch,
       plt_to_hex(plt_cam16_ucs_jmh(50, 40, 120)) AS ucs_jmh;
```

### LMS spaces

| Constructor | Components |
| --- | --- |
| `plt_lms(l, m, s)` | long, medium, short (Bradford matrix, D65) |
| `plt_lms_von_kries(l, m, s)` | long, medium, short (von Kries / Hunt-Pointer-Estevez matrix, D65) |

```sql {"type":"duckfn","show":"table","colors":["lms","von_kries"]}
SELECT plt_to_hex(plt_lms(0.5, 0.4, 0.3)) AS lms,
       plt_to_hex(plt_lms_von_kries(0.5, 0.4, 0.3)) AS von_kries;
```

### One-dimensional luminance spaces

`plt_luma`, `plt_luma_linear` and `plt_luma_gamma` take a single value in 0..1. Both remaining
components are 0. These spaces are lossy: converting a color into one and back gives a gray of the
same luminance, not the original color.

```sql {"type":"duckfn","show":"table","colors":"gray"}
SELECT plt_luma(0.5) AS luma,
       plt_components(plt_convert(plt_srgb(0.2, 0.4, 0.6), 'luma')) AS from_color,
       plt_to_hex(plt_convert(plt_srgb(0.2, 0.4, 0.6), 'luma')) AS gray;
```

## Errors

```sql {"type":"duckfn","expect":"error"}
SELECT plt_color('not-a-space', 1, 2, 3);
```

```sql {"type":"duckfn","expect":"error"}
SELECT plt_component(plt_srgb(1, 0, 0), 4);
```

```sql {"type":"duckfn","expect":"error"}
SELECT plt_from_components('srgb', [1, 2]);
```
