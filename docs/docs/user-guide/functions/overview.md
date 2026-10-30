---
title: Overview
sidebar_position: 1
description: How the 160 plt_ functions are organised, the three rules every function follows, and where to look for each family.
---

# Function reference overview

Loading `duckfn_palette` registers **160 names / 169 signatures**, all under the `plt_` prefix. They
are ordinary DuckDB scalar functions: use them in a projection, a `WHERE` clause, a `GROUP BY`, a
`LATERAL` join or a list lambda, exactly like a built-in function.

## The value they operate on

A color is a single `STRUCT`, so it can live in a column, be compared, grouped and passed to any
other function:

```text
STRUCT(space VARCHAR, c1 DOUBLE, c2 DOUBLE, c3 DOUBLE, alpha DOUBLE)
```

`space` is one of the 44 names in `plt_color_spaces()`; `c1..c3` are that space's components in
palette's `Color::new` order; `alpha` is 0 (transparent) to 1 (opaque) and is carried through
conversions untouched.

```sql {"type":"duckfn","show":"table"}
SELECT plt_color_spaces() AS spaces;
```

## Three rules that apply everywhere

- **NULL in, NULL out.** A NULL in any argument makes the whole call NULL, including a NULL inside
  a `LIST`. Results palette cannot define (an unknown name, a difference with no value) are NULL
  too — never `NaN` and never 0.
- **A bad argument fails the statement.** Unknown space, mode, metric, white point or format names,
  a wrong list length and an out-of-range index raise an error whose message starts with the
  function name, e.g. `plt_convert: unknown color space 'nope'`.
- **Names are normalized.** Space, mode, metric, white-point and format names are case-insensitive,
  and hyphens and spaces count as underscores: `'OKLCH'`, `'ok-lch'` and `' ok lch '` all work.

```sql {"type":"duckfn","show":"table"}
SELECT plt_to_hex(plt_convert(plt_srgb(1, 0.5, 0), 'OK-LCH')) AS normalized;
```

## Where each family lives

| Page | What it covers |
| --- | --- |
| [Colors and spaces](./colors/spaces.md) | The color value, its accessors, alpha, and one constructor per space (44) |
| [Conversion and clamping](./colors/conversion.md) | `plt_convert`, `plt_convert_clamped`, `plt_clamp`, `plt_is_within_bounds`, `plt_equals` |
| [Manipulation](./operations/manipulation.md) | Mixing, lightening, darkening, brightening, greyscale, tint and shade, saturation, hue and premultiplied alpha |
| [Blending and compositing](./operations/blending.md) | 11 blend modes, 6 Porter-Duff operators, custom OpenGL factors |
| [Color difference and contrast](./operations/difference.md) | 7 difference metrics, WCAG 2.1 luminance, ratio and criteria, most-readable picking, YIQ brightness |
| [Color schemes](./operations/theory.md) | Complementary, split-complementary, analogous, triadic, tetradic, monochromatic |
| [Strings, hex and named colors](./formats/strings.md) | Hex (including fixed-width), full CSS Color Level 4, extra output formats, the 148 SVG/CSS3 names, nearest name |
| [8-bit packing](./formats/packing.md) | Pack and unpack ARGB/RGBA/ABGR/BGRA integers |
| [White points, adaptation and encoding](./color-science/encoding.md) | White points, adaptation matrices, transfer functions, primaries, matrices |
| [CAM16 with custom viewing conditions](./color-science/cam16.md) | Full CAM16 attributes and the eight CAM16 spaces under your own conditions |
| [colorous palettes](./palettes/colorous.md) | d3-style gradients and categorical palettes |
| [Random colors](./palettes/random.md) | Reproducible sampling from palette's distributions |

## Finding a function at runtime

```sql {"type":"duckfn","show":"table"}
SELECT function_name, parameters, return_type, description
FROM duckdb_functions()
WHERE function_name LIKE 'plt_%'
ORDER BY function_name;
```

## Two habits that avoid most surprises

**Convert before an operation that needs a particular kind of component.** `plt_saturate` needs a
hue-based space, `plt_blend` an RGB-like one, and the scheme functions a color that carries a hue.
The error message says which function failed and suggests converting first:

```sql {"type":"duckfn","expect":"error"}
SELECT plt_saturate(plt_srgb(1, 0, 0), 0.2);
```

**Work in a perceptually uniform space when mixing or comparing.** `oklab` / `oklch` mix and
compare far more evenly than `srgb`; convert in, do the work, convert back:

```sql {"type":"duckfn","show":"table"}
SELECT plt_to_hex(plt_mix(plt_convert(plt_srgb(1, 0, 0), 'oklab'),
                          plt_convert(plt_srgb(0, 0, 1), 'oklab'),
                          0.5)) AS purple;
```
