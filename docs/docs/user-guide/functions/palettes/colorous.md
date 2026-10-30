---
title: colorous palettes
sidebar_position: 1
description: Sample the 38 colorous gradients at a position or evenly across a list, and take the 10 categorical palettes whole.
---

# colorous palettes

colorous is a port of d3-scale-chromatic: well-known perceptual gradients (viridis, plasma,
spectral, …) and categorical palettes (tableau10, set3, accent, …). Everything here returns
**opaque sRGB** colors; use `plt_convert` if you need another space.

There are two kinds of name and two sets of functions:

| Kind | Count | Functions |
| --- | --- | --- |
| Continuous gradient | 38 | `plt_colorous_gradient`, `plt_colorous_palette` |
| Categorical palette | 10 | `plt_colorous_categorical` |

Names are lowercase constant names from colorous, and matching is case-insensitive with `-` and
spaces counting as `_` (`'Blue Green'` = `'blue_green'`). `grey`/`gray` spellings both work where
colorous itself has them.

## Gradients

### `plt_colorous_gradient(name, t)`

**Signature**: `plt_colorous_gradient(name VARCHAR, t DOUBLE) -> plt_color`

One color from a gradient at position `t` in `[0, 1]`. Out-of-range positions are clamped, so
`-1` gives the start and `2` the end rather than an error.

```sql {"type":"duckfn","show":"table"}
SELECT plt_to_hex(plt_colorous_gradient('viridis', 0)) AS start,
       plt_to_hex(plt_colorous_gradient('viridis', 0.5)) AS middle,
       plt_to_hex(plt_colorous_gradient('viridis', 1)) AS end;
```

```sql {"type":"duckfn","show":"table"}
SELECT t, plt_to_hex(plt_colorous_gradient('spectral', t)) AS color
FROM (VALUES (0.0), (0.25), (0.5), (0.75), (1.0)) v(t);
```

`t` is continuous, so it works as a colormap for any numeric column:

```sql {"type":"duckfn","show":"table"}
SELECT value, plt_to_hex(plt_colorous_gradient('plasma', value / 100.0)) AS color
FROM (VALUES (10), (45), (80)) v(value);
```

### `plt_colorous_palette(name, n)`

**Signature**: `plt_colorous_palette(name VARCHAR, n BIGINT) -> plt_color[]`

`n` colors sampled evenly across the gradient, from start to end. `n` must be at least 1; zero or
a negative count is an error.

```sql {"type":"duckfn","show":"table"}
SELECT to_json(list_transform(plt_colorous_palette('viridis', 5), lambda x: plt_to_hex(x))) AS five,
       length(plt_colorous_palette('viridis', 5)) AS count;
```

This is the usual way to build a categorical scale out of a continuous one — pick as many colors as
you have categories:

```sql {"type":"duckfn","show":"table"}
SELECT i, plt_to_hex(plt_colorous_palette('turbo', 4)[i]) AS color
FROM range(1, 5) t(i);
```

### `plt_colorous_palettes()`

**Signature**: `plt_colorous_palettes() -> VARCHAR[]`

All 38 gradient names — sequential multi-hue (`turbo`, `viridis`, `inferno`, `magma`, `plasma`,
`cividis`, `warm`, `cool`, `cubehelix`, `blue_green`, `blue_purple`, `green_blue`, `orange_red`,
`purple_blue_green`, `purple_blue`, `purple_red`, `red_purple`, `yellow_green_blue`, `yellow_green`,
`yellow_orange_brown`, `yellow_orange_red`), single-hue (`blues`, `greens`, `greys`, `oranges`,
`purples`, `reds`), diverging (`brown_green`, `pink_green`, `purple_green`, `purple_orange`,
`red_blue`, `red_grey`, `red_yellow_blue`, `red_yellow_green`, `spectral`) and cyclical (`rainbow`,
`sinebow`).

```sql {"type":"duckfn","show":"table"}
SELECT list_count(plt_colorous_palettes()) AS count,
       plt_colorous_palettes() AS gradients;
```

## Categorical palettes

### `plt_colorous_categorical(name)`

**Signature**: `plt_colorous_categorical(name VARCHAR) -> plt_color[]`

A whole categorical palette. The length is fixed by the palette: 10 for `category10` and
`tableau10`, 12 for `paired` and `set3`, 9 for `pastel1` and `set1`, and 8 for `accent`, `dark2`,
`pastel2` and `set2`:

```sql {"type":"duckfn","show":"table"}
SELECT to_json(list_transform(plt_colorous_categorical('tableau10'), lambda x: plt_to_hex(x))) AS tableau10;
```

```sql {"type":"duckfn","show":"table"}
SELECT name, length(plt_colorous_categorical(name)) AS colors
FROM unnest(plt_colorous_categoricals()) t(name);
```

### `plt_colorous_categoricals()`

**Signature**: `plt_colorous_categoricals() -> VARCHAR[]`

The 10 categorical names: `category10`, `accent`, `dark2`, `paired`, `pastel1`, `pastel2`, `set1`,
`set2`, `set3`, `tableau10`.

```sql {"type":"duckfn","show":"table"}
SELECT plt_colorous_categoricals() AS palettes;
```

## Errors

An unknown name is a query error that points at the listing function:

```sql {"type":"duckfn","expect":"error"}
SELECT plt_colorous_gradient('viridus', 0.5);
```

```sql {"type":"duckfn","expect":"error"}
SELECT plt_colorous_categorical('tableau20');
```

```sql {"type":"duckfn","expect":"error"}
SELECT plt_colorous_palette('viridis', 0);
```
