---
title: Color schemes
sidebar_position: 4
description: Complementary, split-complementary, analogous, triadic and tetradic schemes, computed by rotating the hue in the color's own space, plus tinycolor-style monochromatic.
---

# Color schemes

A scheme is a set of colors derived from one color by rotating its hue. All of these functions
work **inside the color's own space**: an `oklch` input gives `oklch` results with the same
lightness and chroma and a rotated hue, normalized into 0..360. Alpha is kept.

They need a space that carries a hue. Supported: `hsl`, `hsv`, `hwb`, `hsluv`, `lch`, `lchuv`,
`oklch`, `okhsl`, `okhsv`, `okhwb`, the six partial CAM16 spaces `cam16_jch`, `cam16_jmh`,
`cam16_jsh`, `cam16_qch`, `cam16_qmh`, `cam16_qsh`, and `cam16_ucs_jmh`. A space without a hue is
rejected:

```sql {"type":"duckfn","expect":"error"}
SELECT plt_complementary(plt_lab(50, 20, -30));
```

Converting first is the whole fix:

```sql {"type":"duckfn","show":"table","colors":"complement"}
SELECT plt_to_hex(plt_complementary(plt_convert(plt_srgb(1, 0, 0), 'oklch'))) AS complement;
```

The functions return the scheme's **other** colors, in palette's order and not including the input
color. All of them share the same list shape, so a query can treat them uniformly:

```sql {"type":"duckfn","show":"table","colors":"triadic"}
SELECT list_transform(plt_triadic(plt_oklch(0.6, 0.15, 30)), lambda x: plt_to_hex(x)) AS triadic;
```

## `plt_complementary(color)`

**Signature**: `plt_complementary(color plt_color) -> plt_color`

The color opposite on the hue circle: hue + 180°. This one returns a single color rather than a
list, because a complement is one color.

```sql {"type":"duckfn","show":"table","colors":"complement"}
SELECT plt_to_hex(plt_complementary(plt_hsl(120, 1, 0.5))) AS complement,
       plt_components(plt_complementary(plt_hsl(120, 1, 0.5))) AS complement_hsl;
```

## `plt_split_complementary(color)`

**Signature**: `plt_split_complementary(color plt_color) -> plt_color[]`

The two colors at hue + 150° and + 210° — the neighbors of the complement.

```sql {"type":"duckfn","show":"table"}
SELECT plt_components(plt_split_complementary(plt_hsl(120, 1, 0.5))[1]) AS first,
       plt_components(plt_split_complementary(plt_hsl(120, 1, 0.5))[2]) AS second;
```

## `plt_analogous(color)`

**Signature**: `plt_analogous(color plt_color) -> plt_color[]`

The two colors at hue − 30° and + 30°.

```sql {"type":"duckfn","show":"table","colors":"analogous"}
SELECT list_transform(plt_analogous(plt_hsl(120, 1, 0.5)), lambda x: plt_to_hex(x)) AS analogous;
```

## `plt_analogous_secondary(color)`

**Signature**: `plt_analogous_secondary(color plt_color) -> plt_color[]`

The next pair out, at hue − 60° and + 60°. Together with `plt_analogous` this gives the five-color
analogous scheme.

```sql {"type":"duckfn","show":"table"}
SELECT plt_components(plt_analogous_secondary(plt_hsl(120, 1, 0.5))[1]) AS minus_60,
       plt_components(plt_analogous_secondary(plt_hsl(120, 1, 0.5))[2]) AS plus_60;
```

## `plt_triadic(color)`

**Signature**: `plt_triadic(color plt_color) -> plt_color[]`

The two colors at hue + 120° and + 240°.

```sql {"type":"duckfn","show":"table","colors":"triadic"}
SELECT list_transform(plt_triadic(plt_hsl(120, 1, 0.5)), lambda x: plt_to_hex(x)) AS triadic;
```

## `plt_tetradic(color)`

**Signature**: `plt_tetradic(color plt_color) -> plt_color[]`

The three colors at hue + 90°, + 180° and + 270° — the square scheme.

```sql {"type":"duckfn","show":"table","colors":"tetradic"}
SELECT list_transform(plt_tetradic(plt_hsl(30, 1, 0.5)), lambda x: plt_to_hex(x)) AS tetradic;
```

## `plt_monochromatic(color, count)`

**Signature**: `plt_monochromatic(color plt_color, count INTEGER) -> plt_color[]`

The odd one out: it keeps hue and saturation and cycles HSV **value** by `1/count` from the
current value, wrapping into 0..1. tinycolor's `monochromatic`. It returns exactly `count` colors
and the first is the input color itself — unlike the hue schemes above, which leave the input out.
`count` 0 gives an empty list and a negative count is an error. Results stay in the input space
with the same alpha.

```sql {"type":"duckfn","show":"table","colors":"reds"}
SELECT list_transform(plt_monochromatic(plt_srgb(1, 0, 0), 4),
                      lambda x: plt_to_hex(x)) AS reds,
       plt_components(plt_monochromatic(plt_hsl(120, 1, 0.5), 3)[2]) AS second_hsl;
```

## `plt_harmony(color, scheme)`

**Signature**: `plt_harmony(color plt_color, scheme VARCHAR) -> plt_color[]`

The name-based entry point for all six schemes. It accepts the names in `plt_color_schemes()` plus
the aliases `complement`, `split_complement`, `triad` and `square`. This is what you reach for when
the scheme comes from a column or a parameter. The six schemes all rotate the hue, so
`monochromatic` is not one of them — it varies HSV value instead and has its own function, above.

```sql {"type":"duckfn","show":"table","colors":"colors"}
SELECT scheme,
       list_transform(plt_harmony(plt_oklch(0.6, 0.15, 30), scheme),
                      lambda x: plt_to_hex(x)) AS colors
FROM (VALUES ('complementary'), ('split_complementary'), ('analogous'),
             ('analogous_secondary'), ('triadic'), ('tetradic')) t(scheme);
```

```sql {"type":"duckfn","show":"table","colors":"first_of_square"}
SELECT plt_to_hex(plt_harmony(plt_oklch(0.6, 0.15, 30), 'Square')[1]) AS first_of_square;
```

```sql {"type":"duckfn","expect":"error"}
SELECT plt_harmony(plt_oklch(0.6, 0.15, 30), 'bogus');
```

## `plt_color_schemes()`

**Signature**: `plt_color_schemes() -> VARCHAR[]`

The six canonical scheme names: `complementary`, `split_complementary`, `analogous`,
`analogous_secondary`, `triadic`, `tetradic`.

```sql {"type":"duckfn","show":"table"}
SELECT plt_color_schemes() AS schemes;
```
