---
title: Manipulation
sidebar_position: 1
description: Mix colors, lighten and darken, saturate and desaturate, rotate or set hues, and premultiply alpha — in the color's own space.
---

# Manipulation

These functions change one color. They never convert it: the result keeps the input's space and
alpha, so `plt_lighten(plt_hsl(...), 0.2)` is still an `hsl` value. Each operation is only defined
for the spaces that have the component it acts on — an RGB space has no hue to rotate, and HSL has
no a/b axes — so the supported-space list differs per function.

A space that does not support the operation raises a query error that says which function failed
and suggests converting first:

```sql {"type":"duckfn","expect":"error"}
SELECT plt_saturate(plt_srgb(1, 0, 0), 0.2);
```

The fix is one `plt_convert` away:

```sql {"type":"duckfn","show":"table"}
SELECT plt_to_hex(plt_saturate(plt_convert(plt_srgb(1, 0, 0), 'hsl'), 0.2)) AS saturated;
```

## Mixing

### `plt_mix(a, b, factor)`

**Signature**: `plt_mix(a plt_color, b plt_color, factor DOUBLE) -> plt_color`

Linear interpolation in **`a`'s space**: `factor` 0 gives `a`, 1 gives `b`, 0.5 the midpoint.
`factor` is clamped to 0..1. `b` is converted into `a`'s space first, and alpha is interpolated
along with the components. Mixing is defined in all 44 spaces, including the hue spaces:

```sql {"type":"duckfn","show":"table"}
SELECT plt_to_hex(plt_mix(plt_srgb(1, 0, 0), plt_srgb(0, 0, 1), 0.5)) AS srgb_mid,
       plt_to_hex(plt_mix(plt_convert(plt_srgb(1, 0, 0), 'oklab'),
                          plt_convert(plt_srgb(0, 0, 1), 'oklab'),
                          0.5)) AS oklab_mid;
```

```sql {"type":"duckfn","show":"table"}
SELECT plt_mix(plt_with_alpha(plt_srgb(1, 0, 0), 0),
               plt_with_alpha(plt_srgb(1, 0, 0), 1),
               0.25) AS quarter_alpha;
```

## Lighten and darken

Both variants keep the space. The proportional form (`plt_lighten`, `plt_darken`) scales the
lightness towards the space's maximum or minimum; the fixed form (`plt_lighten_fixed`,
`plt_darken_fixed`) moves it by an absolute amount. Both take a value in 0..1; a value of 0 leaves
the color unchanged.

Supported spaces for all four: all 15 RGB variants, `hsl`, `hsv`, `hwb`, `hsluv`, `lab`, `lch`,
`luv`, `lchuv`, `xyz`, `xyz_d50`, `yxy`, `luma`, `luma_linear`, `luma_gamma`, `oklab`, `oklch`,
`okhsl`, `okhsv`, `okhwb`, `cam16_ucs_jab` and `cam16_ucs_jmh`.

### `plt_lighten(color, factor)`

**Signature**: `plt_lighten(color plt_color, factor DOUBLE) -> plt_color`

```sql {"type":"duckfn","show":"table"}
SELECT plt_to_hex(plt_lighten(plt_srgb(1, 0, 0), 0.2)) AS lighter,
       plt_to_hex(plt_lighten(plt_hsl(0, 1, 0.4), 0.2)) AS lighter_hsl,
       plt_components(plt_lighten(plt_oklch(0.5, 0.1, 30), 0.2)) AS lighter_oklch;
```

### `plt_lighten_fixed(color, amount)`

**Signature**: `plt_lighten_fixed(color plt_color, amount DOUBLE) -> plt_color`

```sql {"type":"duckfn","show":"table"}
SELECT plt_components(plt_lighten_fixed(plt_lab(50, 20, -30), 0.1)) AS raised_lab,
       plt_to_hex(plt_lighten_fixed(plt_hsl(0, 1, 0.4), 0.2)) AS raised_hsl;
```

### `plt_darken(color, factor)`

**Signature**: `plt_darken(color plt_color, factor DOUBLE) -> plt_color`

```sql {"type":"duckfn","show":"table"}
SELECT plt_to_hex(plt_darken(plt_srgb(1, 0, 0), 0.2)) AS darker,
       plt_to_hex(plt_darken(plt_hsv(0, 1, 0.6), 0.3)) AS darker_hsv;
```

### `plt_darken_fixed(color, amount)`

**Signature**: `plt_darken_fixed(color plt_color, amount DOUBLE) -> plt_color`

```sql {"type":"duckfn","show":"table"}
SELECT plt_components(plt_darken_fixed(plt_lab(50, 20, -30), 0.1)) AS lowered_lab;
```

## Saturation

The proportional form scales saturation or chroma towards the space's maximum
(`plt_saturate`) or minimum (`plt_desaturate`); the fixed form adds or subtracts an absolute
amount. The parameter is in 0..1 for both forms.

Supported spaces: `hsl`, `hsv`, `hsluv`, `lch`, `lchuv`, `okhsl`, `okhsv` and `cam16_ucs_jmh`.
`oklch` itself is not in that list — convert to `lch` when you want to scale CIE chroma, or stay in
`okhsl` / `okhsv` when you want to scale perceptual saturation.

### `plt_saturate(color, factor)`

**Signature**: `plt_saturate(color plt_color, factor DOUBLE) -> plt_color`

```sql {"type":"duckfn","show":"table"}
SELECT plt_to_hex(plt_saturate(plt_hsl(0, 0.5, 0.5), 0.5)) AS more_saturated,
       plt_components(plt_saturate(plt_lch(50, 20, 40), 0.5)) AS more_chroma;
```

### `plt_saturate_fixed(color, amount)`

**Signature**: `plt_saturate_fixed(color plt_color, amount DOUBLE) -> plt_color`

```sql {"type":"duckfn","show":"table"}
SELECT plt_components(plt_saturate_fixed(plt_hsl(0, 0.4, 0.5), 0.2)) AS fixed_hsl,
       plt_components(plt_saturate_fixed(plt_okhsl(0, 0.4, 0.5), 0.2)) AS fixed_okhsl;
```

### `plt_desaturate(color, factor)`

**Signature**: `plt_desaturate(color plt_color, factor DOUBLE) -> plt_color`

```sql {"type":"duckfn","show":"table"}
SELECT plt_to_hex(plt_desaturate(plt_hsl(0, 0.8, 0.5), 0.5)) AS muted,
       plt_to_hex(plt_desaturate(plt_convert(plt_srgb(1, 0, 0), 'hsl'), 1)) AS gray;
```

### `plt_desaturate_fixed(color, amount)`

**Signature**: `plt_desaturate_fixed(color plt_color, amount DOUBLE) -> plt_color`

```sql {"type":"duckfn","show":"table"}
SELECT plt_components(plt_desaturate_fixed(plt_hsl(0, 0.8, 0.5), 0.3)) AS less_saturated;
```

## Hue

### `plt_shift_hue(color, degrees)`

**Signature**: `plt_shift_hue(color plt_color, degrees DOUBLE) -> plt_color`

Rotates the hue by any real number of degrees; the result wraps into 0..360.

```sql {"type":"duckfn","show":"table"}
SELECT plt_to_hex(plt_shift_hue(plt_hsl(120, 1, 0.5), 120)) AS shifted,
       plt_to_hex(plt_shift_hue(plt_convert(plt_srgb(1, 0, 0), 'oklch'), -60)) AS backwards;
```

### `plt_with_hue(color, degrees)`

**Signature**: `plt_with_hue(color plt_color, degrees DOUBLE) -> plt_color`

Sets the hue instead of adding to it.

```sql {"type":"duckfn","show":"table"}
SELECT plt_to_hex(plt_with_hue(plt_hsl(120, 1, 0.5), 240)) AS blue,
       plt_components(plt_with_hue(plt_oklch(0.6, 0.1, 30), 200)) AS set_oklch;
```

### `plt_hue(color)`

**Signature**: `plt_hue(color plt_color) -> DOUBLE`

The hue in degrees, 0..360. A gray color reports 0. This works in more spaces than the hue
mutators: every hue space, every RGB space, plus `lab`, `luv` and `oklab`, where the hue is the
angle of the a/b or u/v pair.

```sql {"type":"duckfn","show":"table"}
SELECT plt_hue(plt_srgb(1, 0, 0)) AS red,
       plt_hue(plt_oklab(0.5, 0.1, 0.1)) AS oklab_angle,
       plt_hue(plt_srgb(0.5, 0.5, 0.5)) AS gray;
```

Not every space has a hue; `luma`, `xyz`, `lms` and friends are rejected:

```sql {"type":"duckfn","expect":"error"}
SELECT plt_hue(plt_luma(0.5));
```

## Premultiplied alpha

### `plt_premultiply(color)`

**Signature**: `plt_premultiply(color plt_color) -> plt_color`

Multiplies the components by alpha; alpha itself is unchanged. Alpha must be in 0..1.

```sql {"type":"duckfn","show":"table"}
SELECT plt_premultiply(plt_with_alpha(plt_srgb(1, 0.5, 0.25), 0.5)) AS premultiplied;
```

### `plt_unpremultiply(color)`

**Signature**: `plt_unpremultiply(color plt_color) -> plt_color`

Divides the components by alpha. With alpha 0 the components come back as 0, which is palette's
definition rather than an error.

```sql {"type":"duckfn","show":"table"}
SELECT plt_unpremultiply(plt_premultiply(plt_with_alpha(plt_srgb(1, 0.5, 0.25), 0.5))) AS round_trip,
       plt_unpremultiply(plt_with_alpha(plt_srgb(1, 0.5, 0.25), 0)) AS zero_alpha;
```

Supported spaces for both: all 15 RGB variants, `lab`, `luv`, `xyz`, `xyz_d50`, `yxy`, `luma`,
`luma_linear`, `luma_gamma`, `oklab`, `cam16_ucs_jab`, `lms` and `lms_von_kries`.

```sql {"type":"duckfn","expect":"error"}
SELECT plt_premultiply(plt_with_alpha(plt_srgb(1, 0, 0), 1.5));
```

```sql {"type":"duckfn","expect":"error"}
SELECT plt_premultiply(plt_hsl(120, 0.5, 0.5));
```
