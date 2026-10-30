---
title: Random colors
sidebar_position: 2
description: Sample reproducible random colors inside a color space, one at a time or as a seeded sequence.
---

# Random colors

These functions sample colors from palette's distributions: "uniform inside the space" as palette
defines it for each space (sRGB components over 0..1, Lab `a`/`b` over -128..127, hue around the
full circle, saturation/lightness weighted by the shape of the space's solid, and so on).

Every sampling function takes a **seed**. The same space and seed always produce the same colors,
so results are reproducible across queries and machines, and safe inside `EXPLAIN`.

## `plt_random_color(space, seed)`

**Signature**: `plt_random_color(space VARCHAR, seed BIGINT) -> plt_color`

One color in the given space.

```sql {"type":"duckfn","show":"table","colors":"as_hex"}
SELECT plt_random_color('oklch', 42) AS color,
       plt_to_hex(plt_random_color('oklch', 42)) AS as_hex;
```

The seed is the whole state: change it to get a different color, keep it to get the same one.

```sql {"type":"duckfn","show":"table","colors":"color"}
SELECT seed, plt_to_hex(plt_random_color('srgb', seed)) AS color
FROM range(1, 6) t(seed);
```

## `plt_random_colors(space, count, seed)`

**Signature**: `plt_random_colors(space VARCHAR, count BIGINT, seed BIGINT) -> plt_color[]`

A list of `count` colors drawn from one seeded sequence. Consecutive draws differ from
`plt_random_color` with the same seed, but the sequence itself is fixed for a given
`(space, count, seed)`.

```sql {"type":"duckfn","show":"table","colors":"colors"}
SELECT list_transform(plt_random_colors('srgb', 5, 42), lambda x: plt_to_hex(x)) AS colors,
       length(plt_random_colors('srgb', 5, 42)) AS count;
```

A count of zero gives an empty list; a negative count is an error:

```sql {"type":"duckfn","show":"table"}
SELECT length(plt_random_colors('lab', 0, 7)) AS empty;
```

```sql {"type":"duckfn","expect":"error"}
SELECT plt_random_colors('srgb', -1, 42);
```

This is handy for building test data or a random categorical palette:

```sql {"type":"duckfn","show":"table","colors":"color"}
SELECT i, plt_to_hex(colors[i]) AS color
FROM (SELECT plt_random_colors('srgb', 6, 2026) AS colors) t,
     range(1, 7) r(i);
```

## `plt_random_spaces()`

**Signature**: `plt_random_spaces() -> VARCHAR[]`

The 34 spaces that have a random distribution: the 15 RGB spaces (gamma and linear variants),
`hsl`, `hsv`, `hwb`, `lab`, `lch`, `luv`, `lchuv`, `xyz`, `xyz_d50`, `yxy`, `luma`, `luma_linear`,
`luma_gamma`, `oklab`, `oklch`, `cam16_ucs_jab`, `cam16_ucs_jmh`, `lms` and `lms_von_kries`.

```sql {"type":"duckfn","show":"table"}
SELECT list_count(plt_random_spaces()) AS count,
       plt_random_spaces() AS spaces;
```

Spaces absent from this list — `hsluv`, `okhsl`, `okhsv`, `okhwb`, the partial CAM16 spaces
(`cam16_jch`, `cam16_jmh`, `cam16_jsh`, `cam16_qch`, `cam16_qmh`, `cam16_qsh`) — have no palette
distribution and cannot be sampled:

```sql {"type":"duckfn","expect":"error"}
SELECT plt_random_color('hsluv', 42);
```

```sql {"type":"duckfn","expect":"error"}
SELECT plt_random_color('oklchh', 42);
```

Sample in a space that is listed and convert afterwards if you need another one:

```sql {"type":"duckfn","show":"table"}
SELECT plt_convert(plt_random_color('oklch', 7), 'lch') AS as_lch;
```

## Errors

- An unknown space name, or one without a random distribution, is a query error naming the
  function and listing the supported spaces.
- A negative `count` is a query error.
- NULL in any argument returns NULL.
