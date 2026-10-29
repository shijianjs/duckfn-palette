---
title: Functions
sidebar_position: 3
description: Every SQL function duckfn_palette registers, with examples you can run in the browser.
---

# Functions

Loading the extension registers 145 functions, all under the `plt_` prefix. They behave like DuckDB's
own: use them in a projection, a `WHERE` clause or a `GROUP BY`, and they combine with built-in
functions freely.

## Colors in SQL

A color is a single `STRUCT`: the space tag plus three components plus alpha.

```text
STRUCT(space VARCHAR, c1 DOUBLE, c2 DOUBLE, c3 DOUBLE, alpha DOUBLE)
```

Every space in `plt_color_spaces()` has a constructor of the same name, taking components in palette's
`Color::new` order:

```text
plt_srgb(red, green, blue)
plt_oklch(lightness, chroma, hue)
plt_lab(lightness, a, b)
plt_luma(luma)                          -- one component
```

`plt_color(space, c1, c2, c3)` builds a color from a space *name*, `plt_from_components(space, list)`
takes the three components as a list, and `plt_from_components_alpha` takes a list of four values — the
components plus alpha. `plt_alpha`, `plt_with_alpha` and `plt_opaque` read and write alpha without
touching the color itself.

```sql {"type":"duckfn","show":"table"}
SELECT plt_color_spaces() AS spaces;
```

```sql {"type":"duckfn","show":"table"}
SELECT plt_srgb(1, 0.5, 0) AS color,
       plt_space(plt_srgb(1, 0.5, 0)) AS space,
       plt_components(plt_srgb(1, 0.5, 0)) AS components,
       plt_alpha(plt_with_alpha(plt_srgb(1, 0.5, 0), 0.5)) AS alpha;
```

```sql {"type":"duckfn","show":"table"}
SELECT plt_to_hex(plt_from_components('srgb', [0.2, 0.4, 0.6])) AS built;
```

```sql {"type":"duckfn","show":"table"}
SELECT plt_to_hex(plt_from_components_alpha('srgb', [0.2, 0.4, 0.6, 0.5])) AS translucent;
```

## Conversion

`plt_convert(color, space)` moves a color into another space and keeps the alpha. The result carries the
target space's tag, so it can be converted again. `plt_convert_clamped` additionally clamps the result
into the target space's standard range, `plt_clamp` clamps in the current space, and
`plt_is_within_bounds` reports whether clamping would change anything.

```sql {"type":"duckfn","show":"table"}
SELECT plt_to_hex(plt_srgb(1, 0.5, 0)) AS srgb,
       plt_to_hex(plt_convert(plt_srgb(1, 0.5, 0), 'oklch')) AS as_oklch,
       plt_to_hex(plt_convert(plt_srgb(1, 0.5, 0), 'xyz')) AS as_xyz;
```

```sql {"type":"duckfn","show":"table"}
SELECT plt_lab(50, 200, 200) AS out_of_range,
       plt_is_within_bounds(plt_lab(50, 200, 200)) AS within_bounds,
       plt_components(plt_clamp(plt_lab(50, 200, 200))) AS clamped;
```

## Lightening, darkening, saturation and hue

These operations are only defined for some spaces, because they act on a specific kind of component.
An RGB space supports `plt_lighten` / `plt_darken`; saturation wants a hue-based space (`hsl`, `hsv`,
`lch`, `okhsl`, …), and the hue operations want one that carries a hue. Converting first is the usual
pattern:

```sql {"type":"duckfn","show":"table"}
SELECT plt_to_hex(plt_lighten(plt_srgb(1, 0, 0), 0.2)) AS lighter,
       plt_to_hex(plt_darken(plt_srgb(1, 0, 0), 0.2)) AS darker;
```

```sql {"type":"duckfn","show":"table"}
SELECT plt_to_hex(plt_saturate(plt_convert(plt_srgb(1, 0, 0), 'hsl'), 0.2)) AS saturated,
       plt_to_hex(plt_shift_hue(plt_convert(plt_srgb(1, 0, 0), 'oklch'), 120)) AS shifted;
```

`plt_mix(a, b, factor)` interpolates two colors (in `a`'s space); `plt_premultiply` /
`plt_unpremultiply` convert between straight and premultiplied alpha.

```sql {"type":"duckfn","show":"table"}
SELECT plt_to_hex(plt_mix(plt_srgb(1, 0, 0), plt_srgb(0, 0, 1), 0.5)) AS purple;
```

## Blending and compositing

`plt_blend(source, destination, mode)` applies one of 11 separable blend modes, and
`plt_compose(source, destination, mode)` one of 6 Porter-Duff operators. The first argument is the
source and the second the destination, as in palette. Custom OpenGL-style factors go through
`plt_blend_equations`.

```sql {"type":"duckfn","show":"table"}
SELECT plt_blend_modes() AS blend_modes, plt_compose_modes() AS compose_modes;
```

```sql {"type":"duckfn","show":"table"}
SELECT plt_to_hex(plt_blend(plt_srgb(0.5, 0.2, 0.1), plt_srgb(0.4, 0.6, 0.8), 'multiply')) AS multiplied;
SELECT plt_to_hex(plt_compose(plt_with_alpha(plt_srgb(1, 0, 0), 0.5),
                              plt_with_alpha(plt_srgb(0, 0, 1), 0.5), 'over')) AS composited;
```

## Color difference and contrast

`plt_difference(a, b, metric)` computes a color difference; with a fourth argument the metric is
computed in that space instead of CIE L\*a\*b\*. `plt_contrast` is the WCAG 2.1 ratio (1 to 21) and
`plt_contrast_check` answers the WCAG criteria directly.

```sql {"type":"duckfn","show":"table"}
SELECT plt_difference_metrics() AS metrics, plt_contrast_criteria() AS criteria;
```

```sql {"type":"duckfn","show":"table"}
SELECT plt_difference(plt_lab(50, 2.6772, -79.7751), plt_lab(50, 0, -82.7485), 'ciede2000') AS delta_e;
```

```sql {"type":"duckfn","show":"table"}
SELECT plt_contrast(plt_srgb(1, 1, 1), plt_srgb(0, 0, 0)) AS contrast,
       plt_contrast_check(plt_srgb(1, 1, 1), plt_srgb(0, 0, 0), 'min_text') AS passes;
```

## Color schemes

The scheme functions need a color with a hue — convert to `oklch` (or any other hue space) first.
`plt_complementary`, `plt_split_complementary`, `plt_analogous`, `plt_triadic` and `plt_tetradic` return
lists of colors; `plt_harmony(color, scheme)` picks one by name.

```sql {"type":"duckfn","show":"table"}
SELECT plt_color_schemes() AS schemes;
```

```sql {"type":"duckfn","show":"table"}
SELECT plt_to_hex(plt_complementary(plt_convert(plt_srgb(1, 0, 0), 'oklch'))) AS complement;
```

```sql {"type":"duckfn","show":"table"}
SELECT to_json(list_transform(plt_harmony(plt_convert(plt_srgb(1, 0, 0), 'oklch'), 'triadic'),
                              lambda x: plt_to_hex(x))) AS triadic;
```

## Named colors, CSS and hex

`plt_named(name)` looks up one of the CSS named colors, `plt_from_hex` parses hex, `plt_to_hex`
serializes to the shortest hex form, and `plt_css_parse` / `plt_css_serialize` handle full CSS color
syntax in either direction. `plt_css_is_valid` answers whether a string parses, and `plt_css_formats()`
lists the serialization formats.

```sql {"type":"duckfn","show":"table"}
SELECT plt_named_names() AS names;
```

```sql {"type":"duckfn","show":"table"}
SELECT plt_to_hex(plt_named('rebeccapurple')) AS named,
       plt_to_hex(plt_from_hex('#ff8000')) AS from_hex,
       plt_to_hex(plt_css_parse('rgb(255 128 0 / 50%)')) AS parsed;
```

```sql {"type":"duckfn","show":"table"}
SELECT plt_css_formats() AS formats,
       plt_css_serialize(plt_srgb(1, 0.5, 0), 'hex') AS as_hex,
       plt_css_serialize(plt_srgb(1, 0.5, 0), 'rgb') AS as_rgb;
```

## White points, chromatic adaptation, encoding and matrices

`plt_white_points()` lists the named white points; `plt_white_point_xyz` / `plt_white_point_xy` return
their chromaticity. `plt_chromatic_adapt(color, target, method)` adapts a color between white points
(with a 4-argument overload for an explicit source), and the matrix functions expose the pieces it is
built from.

```sql {"type":"duckfn","show":"table"}
SELECT plt_white_points() AS white_points;
```

```sql {"type":"duckfn","show":"table"}
SELECT plt_adaptation_methods() AS methods;
```

```sql {"type":"duckfn","show":"table"}
SELECT plt_to_hex(plt_chromatic_adapt(plt_xyz(0.4, 0.3, 0.2), 'd50', 'bradford')) AS adapted;
SELECT plt_adaptation_matrix('d65', 'd50', 'bradford')[1] AS first_entry;
```

`plt_encoding_spaces()` lists the RGB spaces with an encoding, and each encoding exposes its transfer
function, white point and primaries. `plt_decode` / `plt_encode` apply the transfer function by name.

```sql {"type":"duckfn","show":"table"}
SELECT plt_transfer_function('srgb') AS transfer,
       plt_white_point_of('srgb') AS white_point,
       plt_decode('srgb', 0.5) AS decoded,
       plt_encode('srgb', 0.5) AS encoded;
```

```sql {"type":"duckfn","show":"table"}
SELECT plt_rgb_primaries('srgb') AS primaries,
       plt_rgb_to_xyz_matrix('srgb')[1] AS matrix_entry;
```

Matrices are nine doubles in row-major order. `plt_matrix_multiply(a, b)` applies `a` first and then
`b`; `plt_apply_matrix` applies a matrix to a color's components without changing its space.

```sql {"type":"duckfn","show":"table"}
SELECT plt_matrix_identity() AS identity;
```

```sql {"type":"duckfn","show":"table"}
SELECT plt_matrix_apply(plt_matrix_scale(2, 2, 2), 1, 2, 3) AS scaled;
```

## 8-bit packing

`plt_pack(color)` quantizes to 8-bit sRGB and packs into a 32-bit integer in ARGB order;
`plt_pack(color, order)` and `plt_unpack(value, order)` take the order as an argument. The orders are
`argb`, `rgba`, `abgr` and `bgra`.

```sql {"type":"duckfn","show":"table"}
SELECT plt_pack(plt_srgb(1, 0.5, 0)) AS packed,
       plt_to_hex(plt_unpack(plt_pack(plt_srgb(1, 0.5, 0)))) AS unpacked;
```

## CAM16 with custom viewing conditions

The `cam16_*` spaces in `plt_convert` always use palette's default viewing conditions. The
`plt_cam16_*` functions make them an argument: `plt_cam16_params` builds the five parameters from a
named white point, `plt_cam16_params_xyz` from a numeric one, and the `from_xyz` / `to_xyz` pairs
convert between `xyz` and the full CAM16 attributes or a partial CAM16 space.

```sql {"type":"duckfn","show":"table"}
SELECT plt_cam16_surrounds() AS surrounds, plt_cam16_spaces() AS spaces;
```

```sql {"type":"duckfn","show":"table"}
SELECT plt_cam16_from_xyz(plt_xyz(0.4, 0.3, 0.2), plt_cam16_default_params()).lightness AS lightness,
       plt_to_hex(plt_cam16_to_xyz(plt_cam16_from_xyz(plt_xyz(0.4, 0.3, 0.2),
                                                     plt_cam16_default_params()),
                                   plt_cam16_default_params())) AS round_trip;
```

## colorous presets

colorous ships d3-style gradients and categorical palettes. `plt_colorous_gradient` samples a gradient
at a position in `[0, 1]`, `plt_colorous_palette` samples `n` evenly spaced colors, and the
`categorical` pair returns whole palettes.

```sql {"type":"duckfn","show":"table"}
SELECT plt_to_hex(plt_colorous_gradient('viridis', 0.5)) AS midpoint,
       to_json(list_transform(plt_colorous_palette('viridis', 3), lambda x: plt_to_hex(x))) AS sampled;
```

```sql {"type":"duckfn","show":"table"}
SELECT to_json(list_transform(plt_colorous_categorical('tableau10'), lambda x: plt_to_hex(x))) AS tableau10;
```

## Random colors

`plt_random_color(space, seed)` samples a color from palette's uniform distribution for that space,
and `plt_random_colors(space, n, seed)` returns a sequence. The same seed always gives the same
result; `plt_random_spaces()` lists the spaces that have a distribution.

```sql {"type":"duckfn","show":"table"}
SELECT plt_to_hex(plt_random_colors('oklch', 3, 42)[1]) AS first,
       length(plt_random_colors('oklch', 3, 42)) AS count;
```

## Notes

- **NULL in, NULL out.** Any NULL argument gives NULL, and so does anything palette cannot compute —
  never `NaN` or 0.
- **A rejected argument fails the statement.** The error names the function, for example
  `plt_convert: unknown color space 'nope'`.

```sql {"type":"duckfn","expect":"error"}
SELECT plt_convert(plt_srgb(1, 0, 0), 'nope');
```

- **Names are normalized**: case-insensitive, with hyphens and spaces treated as underscores.

```sql {"type":"duckfn","show":"table"}
SELECT plt_to_hex(plt_convert(plt_srgb(1, 0.5, 0), 'OKLCH')) AS upper_case;
```
