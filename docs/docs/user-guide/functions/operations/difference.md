---
title: Color difference and contrast
sidebar_position: 3
description: Seven color difference metrics in the spaces where they are defined, plus WCAG 2.1 relative luminance, contrast ratios, pass/fail checks, most-readable picking and YIQ brightness.
---

# Color difference and contrast

Two families answer different questions. A **color difference** is one number for "how far apart do
these two colors look", and the metric decides the space it is measured in. **WCAG contrast** is a
regulatory ratio for text and graphics, always computed in sRGB.

## `plt_difference(a, b, metric)` and `plt_difference(a, b, metric, space)`

**Signature**: `plt_difference(a plt_color, b plt_color, metric VARCHAR) -> DOUBLE`
**Signature**: `plt_difference(a plt_color, b plt_color, metric VARCHAR, space VARCHAR) -> DOUBLE`

The three-argument form measures in CIE L\*a\*b\* (D65) — the space ΔE, HyAb and CIEDE2000 are
defined in. The four-argument form converts **both** colors into `space` (without clamping) and
measures there; if the metric has no definition in that space the statement fails and the message
lists the spaces that do work.

```sql {"type":"duckfn","show":"table"}
SELECT plt_difference(plt_lab(50, 2.6772, -79.7751), plt_lab(50, 0, -82.7485), 'ciede2000') AS delta_e,
       plt_difference(plt_lab(50, 2.6772, -79.7751), plt_lab(50, 0, -82.7485), 'ciede2000', 'lch') AS same_in_lch;
```

| Metric | Meaning | Defined in |
| --- | --- | --- |
| `euclidean` | straight-line distance in the space's components | `cam16_ucs_jab`, `lab`, `lms`, `lms_von_kries`, `luma`, `luma_linear`, `luma_gamma`, `luv`, `oklab`, every RGB space, `xyz`, `xyz_d50`, `yxy` |
| `euclidean_squared` | the same, squared (cheaper to sort, same ordering) | same as `euclidean` |
| `delta_e` | CIE76 ΔE\*ab | `cam16_ucs_jab`, `cam16_ucs_jmh`, `lab`, `lch` |
| `improved_delta_e` | ΔE\*ab with palette's correction | `cam16_ucs_jab`, `cam16_ucs_jmh`, `lab`, `lch` |
| `hyab` | hybrid distance: Lab lightness difference plus a/b (or u/v) distance | `cam16_ucs_jab`, `lab`, `luv`, `oklab` |
| `ciede2000` | CIEDE2000 ΔE\*00, the standard perceptual metric | `lab`, `lch` |
| `improved_ciede2000` | CIEDE2000 with palette's discontinuity fix | `lab`, `lch` |

Aliases: `cie76`, `deltae` and `delta_e_76` mean `delta_e`; `cie2000` and `delta_e_2000` mean
`ciede2000`. Names are case-insensitive with `-` and spaces counting as `_`.

```sql {"type":"duckfn","show":"table"}
SELECT plt_difference_metrics() AS metrics;
```

```sql {"type":"duckfn","show":"table"}
SELECT plt_difference(plt_srgb(1, 0, 0), plt_srgb(0, 1, 0), 'euclidean', 'oklab') AS oklab_distance,
       plt_difference(plt_srgb(1, 0, 0), plt_srgb(0, 1, 0), 'euclidean', 'srgb') AS srgb_distance,
       plt_difference(plt_srgb(1, 0, 0), plt_srgb(1, 0.05, 0), 'delta_e') AS nearly_identical;
```

A metric measured in the wrong space is an error, not a silent conversion:

```sql {"type":"duckfn","expect":"error"}
SELECT plt_difference(plt_srgb(1, 0, 0), plt_srgb(0, 1, 0), 'ciede2000', 'srgb');
```

```sql {"type":"duckfn","expect":"error"}
SELECT plt_difference(plt_srgb(1, 0, 0), plt_srgb(0, 1, 0), 'lab-distance');
```

## `plt_relative_luminance(color)`

**Signature**: `plt_relative_luminance(color plt_color) -> DOUBLE`

The WCAG 2.1 relative luminance: 0 for black, 1 for white. The color is converted to sRGB first,
because the definition assumes sRGB. This is the quantity the contrast ratio is built from, and
useful on its own for deciding whether a color is "light" or "dark":

```sql {"type":"duckfn","show":"table"}
SELECT plt_relative_luminance(plt_srgb(0, 0, 0)) AS black,
       plt_relative_luminance(plt_srgb(0.5, 0.5, 0.5)) AS middle_gray,
       plt_relative_luminance(plt_srgb(1, 1, 1)) AS white,
       plt_relative_luminance(plt_lab(50, 0, 0)) AS lab_gray;
```

## `plt_contrast(a, b)`

**Signature**: `plt_contrast(a plt_color, b plt_color) -> DOUBLE`

The WCAG 2.1 contrast ratio `(L_lighter + 0.05) / (L_darker + 0.05)`, from 1 (identical) to 21
(black against white). Both colors are converted to sRGB. The result is symmetric:

```sql {"type":"duckfn","show":"table"}
SELECT plt_contrast(plt_srgb(1, 1, 1), plt_srgb(0, 0, 0)) AS max_contrast,
       plt_contrast(plt_srgb(1, 0, 0), plt_srgb(1, 1, 1)) AS red_on_white,
       plt_contrast(plt_srgb(1, 0, 0), plt_srgb(1, 0, 0)) AS identical;
```

```sql {"type":"duckfn","show":"table"}
SELECT plt_contrast(plt_srgb(1, 1, 1), plt_srgb(0, 0, 0)) =
       plt_contrast(plt_srgb(0, 0, 0), plt_srgb(1, 1, 1)) AS symmetric;
```

## `plt_contrast_check(a, b, criterion)`

**Signature**: `plt_contrast_check(a plt_color, b plt_color, criterion VARCHAR) -> BOOLEAN`

Answers a WCAG criterion directly, using the same ratio:

| Criterion | Minimum ratio |
| --- | --- |
| `min_text` | 4.5 |
| `min_large_text` | 3 |
| `enhanced_text` | 7 |
| `enhanced_large_text` | 4.5 |
| `min_graphics` | 3 |

```sql {"type":"duckfn","show":"table"}
SELECT plt_contrast_criteria() AS criteria;
```

```sql {"type":"duckfn","show":"table"}
SELECT plt_contrast_check(plt_srgb(1, 1, 1), plt_srgb(0, 0, 0), 'min_text') AS white_on_black,
       plt_contrast_check(plt_srgb(1, 1, 1), plt_srgb(1, 0, 0), 'min_text') AS white_on_red,
       plt_contrast_check(plt_srgb(1, 1, 1), plt_srgb(1, 0, 0), 'min_large_text') AS white_on_red_large;
```

A useful pattern is to annotate a palette and filter by the criteria it must meet:

```sql {"type":"duckfn","show":"table","colors":"color"}
SELECT plt_to_hex(c) AS color,
       plt_contrast(plt_srgb(1, 1, 1), c) AS on_white,
       plt_contrast_check(plt_srgb(1, 1, 1), c, 'min_text') AS readable_on_white
FROM (VALUES (plt_named('rebeccapurple')),
             (plt_named('gold')),
             (plt_named('teal'))) t(c);
```

```sql {"type":"duckfn","expect":"error"}
SELECT plt_contrast_check(plt_srgb(1, 1, 1), plt_srgb(0, 0, 0), 'aaa');
```

## `plt_most_readable(base, candidates)` and overloads

**Signature**: `plt_most_readable(base plt_color, candidates plt_color[]) -> plt_color`
**Signature**: `plt_most_readable(base plt_color, candidates plt_color[], criterion VARCHAR) -> plt_color`
**Signature**: `plt_most_readable(base plt_color, candidates plt_color[], criterion VARCHAR, use_black_and_white BOOLEAN) -> plt_color`

Picks the candidate with the highest WCAG contrast against `base`, comparing the same ratio
`plt_contrast` returns. The default criterion is `min_text`; it is only consulted when
`use_black_and_white` is true. In that case, if the best candidate still misses the criterion,
the function returns whichever of black and white contrasts better with `base` — tinycolor's
`mostReadable` behaviour. An empty candidate list is an error, and an unknown criterion fails the
statement like `plt_contrast_check`.

```sql {"type":"duckfn","show":"table","colors":["best","fallback"]}
SELECT plt_to_hex(plt_most_readable(plt_named('navy'),
                                    [plt_named('teal'), plt_named('gold'), plt_named('tomato')])) AS best,
       plt_to_hex(plt_most_readable(plt_named('navy'),
                                    [plt_named('teal')], 'min_text', true)) AS fallback;
```

This is the "give me a label color that is readable on this background" query, applied per row:

```sql {"type":"duckfn","show":"table","colors":["background","label"]}
SELECT plt_to_hex(bg) AS background,
       plt_to_hex(plt_most_readable(bg, [plt_named('white'), plt_named('black')])) AS label
FROM (VALUES (plt_named('seagreen')), (plt_named('gold')), (plt_named('rebeccapurple'))) t(bg);
```

```sql {"type":"duckfn","expect":"error"}
SELECT plt_most_readable(plt_srgb(1, 1, 1), []);
```

## Brightness and light/dark

`plt_relative_luminance` follows WCAG, which is what contrast ratios use. The three functions below
use the older **YIQ** brightness instead — the value tinycolor exposes as `getBrightness`. It is
cruder and channel-weighted (0.299 R + 0.587 G + 0.114 B), but it is what many design tools use to
decide whether text should be light or dark.

### `plt_brightness(color)`

**Signature**: `plt_brightness(color plt_color) -> DOUBLE`

The YIQ brightness, 0 (black) to 1 (white). The color is converted to sRGB and clamped first.

```sql {"type":"duckfn","show":"table"}
SELECT plt_brightness(plt_srgb(0, 0, 0)) AS black,
       plt_brightness(plt_srgb(0.5, 0.5, 0.5)) AS middle_gray,
       plt_brightness(plt_srgb(1, 1, 1)) AS white,
       plt_brightness(plt_named('gold')) AS gold;
```

### `plt_is_light(color)` and `plt_is_dark(color)`

**Signature**: `plt_is_light(color plt_color) -> BOOLEAN`
**Signature**: `plt_is_dark(color plt_color) -> BOOLEAN`

A threshold on the same YIQ value at 128/255 (≈ 0.502), tinycolor's default. The two are exact
complements — a color is either light or dark, never both and never neither.

```sql {"type":"duckfn","show":"table"}
SELECT plt_is_light(plt_named('gold')) AS gold_light,
       plt_is_dark(plt_named('gold')) AS gold_dark,
       plt_is_light(plt_named('navy')) AS navy_light,
       plt_is_dark(plt_named('navy')) AS navy_dark;
```

## Listing the supported names

### `plt_difference_metrics()`

**Signature**: `plt_difference_metrics() -> VARCHAR[]`

The seven metric names accepted by `plt_difference`.

### `plt_contrast_criteria()`

**Signature**: `plt_contrast_criteria() -> VARCHAR[]`

The five criterion names accepted by `plt_contrast_check`.
