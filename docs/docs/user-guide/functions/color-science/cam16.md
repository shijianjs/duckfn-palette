---
title: CAM16 with custom viewing conditions
sidebar_position: 2
description: Build CAM16 viewing conditions from a named or numeric white point and convert xyz to the full CAM16 attributes or any of the eight CAM16 spaces.
---

# CAM16 with custom viewing conditions

`plt_convert` can move a color into any of the eight `cam16_*` spaces, but it always uses palette's
default viewing conditions: D65, adapting luminance 40, background luminance 0.2, an average
surround, and automatic discounting. That is the right choice for "just convert it".

This page is for the other case: you know the conditions the color is viewed under and want them
to be part of the math. The workflow is always the same:

1. build a `plt_cam16_params` value with `plt_cam16_params` or `plt_cam16_params_xyz`;
2. convert between `xyz` and CAM16 with that value.

The functions only accept `xyz`. If your color is anywhere else, convert it first and convert the
result back afterwards.

```sql {"type":"duckfn","show":"table"}
SELECT plt_cam16_from_xyz(plt_convert(plt_srgb(1, 0.5, 0), 'xyz'),
                          plt_cam16_default_params()) AS attributes;
```

## Building the conditions

### `plt_cam16_default_params()`

**Signature**: `plt_cam16_default_params() -> plt_cam16_params`

Palette's defaults, as a value you can pass on. Using it gives exactly the same numbers as
`plt_convert(..., 'cam16_jch')`:

```sql {"type":"duckfn","show":"table"}
SELECT plt_cam16_default_params() AS params;
```

### `plt_cam16_params(white_point, adapting_luminance, background_luminance, surround)` and `plt_cam16_params(white_point, adapting_luminance, background_luminance, surround, discounting)`

**Signature**: `plt_cam16_params(white_point VARCHAR, adapting_luminance DOUBLE, background_luminance DOUBLE, surround VARCHAR) -> plt_cam16_params`
**Signature**: `plt_cam16_params(white_point VARCHAR, adapting_luminance DOUBLE, background_luminance DOUBLE, surround VARCHAR, discounting DOUBLE) -> plt_cam16_params`

The white point is one of the names in `plt_white_points()`. The two luminances are in `cd/m²` and
`0..1` respectively (palette's `La` and `Yb`). `surround` is `dark`, `dim`, `average` (0%, 10%,
20% field of view) or a percentage written as text such as `'12.5'`.

Without the fifth argument, discounting is **automatic** — palette derives it from the luminance and
surround. With it, discounting is the given 0..1 factor:

```sql {"type":"duckfn","show":"table"}
SELECT plt_cam16_params('d65', 40, 0.2, 'average') AS defaults,
       plt_cam16_params('d50', 100, 0.2, 'dim') AS print_viewing,
       plt_cam16_params('d65', 40, 0.2, '12.5') AS custom_surround,
       plt_cam16_params('d65', 40, 0.2, 'dim', 0.9) AS fixed_discounting;
```

### `plt_cam16_params_xyz(white_x, white_y, white_z, adapting_luminance, background_luminance, surround)` and `plt_cam16_params_xyz(white_x, white_y, white_z, adapting_luminance, background_luminance, surround, discounting)`

**Signature**: `plt_cam16_params_xyz(white_x DOUBLE, white_y DOUBLE, white_z DOUBLE, adapting_luminance DOUBLE, background_luminance DOUBLE, surround VARCHAR) -> plt_cam16_params`
**Signature**: `plt_cam16_params_xyz(white_x DOUBLE, white_y DOUBLE, white_z DOUBLE, adapting_luminance DOUBLE, background_luminance DOUBLE, surround VARCHAR, discounting DOUBLE) -> plt_cam16_params`

The same, with the white point given as three numbers — palette's dynamic white point. This is for
non-standard illuminants that have no name:

```sql {"type":"duckfn","show":"table"}
SELECT plt_cam16_params_xyz(0.95047, 1, 1.08883, 40, 0.2, 'average') AS same_as_d65,
       plt_cam16_params_xyz(0.94, 1, 1.05, 40, 0.2, 'average', 0.8) AS custom_white;
```

### `plt_cam16_surrounds()`

**Signature**: `plt_cam16_surrounds() -> VARCHAR[]`

The three preset names and their percentages: `dark` (0%), `dim` (10%), `average` (20%). A numeric
percentage is accepted anywhere a name is.

```sql {"type":"duckfn","show":"table"}
SELECT plt_cam16_surrounds() AS surrounds;
```

## Converting

### `plt_cam16_from_xyz(color, params)`

**Signature**: `plt_cam16_from_xyz(color plt_color, params plt_cam16_params) -> plt_cam16_full`

Converts an `xyz` color into the **full** set of CAM16 attributes, as a seven-field struct:

```text
STRUCT(lightness, chroma, hue, brightness, colorfulness, saturation, alpha)
```

`J`, `C`, `h` are the lightness, chroma and hue; `Q`, `M`, `s` the brightness, colorfulness and
saturation. Alpha is copied from the input.

```sql {"type":"duckfn","show":"table"}
SELECT plt_cam16_from_xyz(plt_xyz(0.4, 0.3, 0.2), plt_cam16_default_params()) AS attrs;
```

The conditions matter: the same `xyz` under different conditions gives different attributes.

```sql {"type":"duckfn","show":"table"}
SELECT (plt_cam16_from_xyz(c, plt_cam16_params('d65', 40, 0.2, 'average'))).lightness AS average,
       (plt_cam16_from_xyz(c, plt_cam16_params('d65', 40, 0.2, 'dark'))).lightness AS dark,
       (plt_cam16_from_xyz(c, plt_cam16_params('d65', 200, 0.2, 'average'))).lightness AS bright
FROM (SELECT plt_xyz(0.4, 0.3, 0.2) AS c);
```

### `plt_cam16_to_xyz(cam16, params)`

**Signature**: `plt_cam16_to_xyz(cam16 plt_cam16_full, params plt_cam16_params) -> plt_color`

The inverse, using the same conditions. Palette assumes the attributes are mutually consistent and
uses only some of them when it converts back; the result is an `xyz` color with the alpha of the
full struct.

```sql {"type":"duckfn","show":"table"}
SELECT plt_to_hex(plt_convert(plt_cam16_to_xyz(plt_cam16_from_xyz(plt_xyz(0.4, 0.3, 0.2),
                                                                 plt_cam16_default_params()),
                                               plt_cam16_default_params()), 'srgb')) AS round_trip;
```

### `plt_cam16_partial_from_xyz(color, space, params)`

**Signature**: `plt_cam16_partial_from_xyz(color plt_color, space VARCHAR, params plt_cam16_params) -> plt_color`

Converts an `xyz` color into one of the eight CAM16 spaces, with your conditions. The result is a
normal `plt_color` whose `space` is the requested CAM16 name, so it can be converted further with
`plt_convert` (which then uses the default conditions again — the conditions are not carried in the
value):

```sql {"type":"duckfn","show":"table"}
SELECT plt_cam16_spaces() AS spaces;
```

```sql {"type":"duckfn","show":"table"}
SELECT plt_cam16_partial_from_xyz(plt_xyz(0.4, 0.3, 0.2), 'cam16_jch',
                                  plt_cam16_default_params()) AS jch,
       plt_cam16_partial_from_xyz(plt_xyz(0.4, 0.3, 0.2), 'cam16_ucs_jmh',
                                  plt_cam16_default_params()) AS ucs_jmh;
```

```sql {"type":"duckfn","show":"table"}
SELECT space,
       plt_components(plt_cam16_partial_from_xyz(plt_xyz(0.4, 0.3, 0.2), space,
                                                 plt_cam16_default_params())) AS components
FROM unnest(plt_cam16_spaces()) t(space);
```

### `plt_cam16_partial_to_xyz(color, params)`

**Signature**: `plt_cam16_partial_to_xyz(color plt_color, params plt_cam16_params) -> plt_color`

Converts any of the eight CAM16 spaces back to `xyz` under the given conditions. The color must
already be in a CAM16 space; there is no implicit conversion, so a wrong space is an error:

```sql {"type":"duckfn","show":"table"}
SELECT plt_components(plt_cam16_partial_to_xyz(
           plt_cam16_partial_from_xyz(plt_xyz(0.4, 0.3, 0.2), 'cam16_jmh',
                                      plt_cam16_default_params()),
           plt_cam16_default_params())) AS round_trip;
```

```sql {"type":"duckfn","expect":"error"}
SELECT plt_cam16_partial_to_xyz(plt_srgb(1, 0, 0), plt_cam16_default_params());
```

```sql {"type":"duckfn","expect":"error"}
SELECT plt_cam16_partial_from_xyz(plt_xyz(0.4, 0.3, 0.2), 'oklch', plt_cam16_default_params());
```

### `plt_cam16_spaces()`

**Signature**: `plt_cam16_spaces() -> VARCHAR[]`

The eight CAM16 space names: `cam16_jch`, `cam16_jmh`, `cam16_jsh`, `cam16_qch`, `cam16_qmh`,
`cam16_qsh`, `cam16_ucs_jab`, `cam16_ucs_jmh`.

## Defaults versus custom conditions

| Task | Use |
| --- | --- |
| Convert a color to CAM16 for comparison or storage | `plt_convert(color, 'cam16_ucs_jab')` |
| Compare CAM16 distances | `plt_difference(a, b, 'delta_e', 'cam16_ucs_jab')` |
| Model a specific display, room or illuminant | the `plt_cam16_*` functions on this page |
| Inspect all six attributes at once | `plt_cam16_from_xyz` |

The first two use palette's defaults and are the cheap path. The last two are the ones where you
pass a `plt_cam16_params` value around — typically built once and reused for a whole table:

```sql {"type":"duckfn","show":"table"}
SELECT plt_to_hex(plt_convert(plt_cam16_partial_to_xyz(
           plt_cam16_partial_from_xyz(plt_convert(c, 'xyz'), 'cam16_jch', p), p), 'srgb')) AS same_color
FROM (SELECT plt_srgb(0.2, 0.4, 0.6) AS c,
             plt_cam16_params('d65', 40, 0.2, 'average') AS p);
```
