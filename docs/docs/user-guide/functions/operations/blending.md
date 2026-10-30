---
title: Blending and compositing
sidebar_position: 2
description: Eleven separable blend modes, six Porter-Duff operators and custom OpenGL-style factors, with the source/destination order palette uses.
---

# Blending and compositing

All three functions take **source first, destination second** — the same order as palette's
`source.multiply(destination)`. If the second color is in a different space it is converted into the
first color's space first, and the result carries that space.

Because the modes are defined per component, they need an RGB-like stimulus space. Which spaces are
accepted differs per function:

| Function | Spaces accepted as the first argument |
| --- | --- |
| `plt_blend` | 15 RGB spaces, `xyz`, `xyz_d50`, `luma`, `luma_linear`, `luma_gamma`, `lms`, `lms_von_kries` |
| `plt_compose` | the same, plus `lab`, `luv`, `yxy`, `oklab`, `cam16_ucs_jab` |
| `plt_blend_equations` | the same as `plt_compose` |

Hue spaces (`hsl`, `lch`, `oklch`, CAM16 JCH/QCH/… ) are rejected — blending a hue angle
component-wise has no meaning. Convert first:

```sql {"type":"duckfn","expect":"error"}
SELECT plt_blend(plt_hsl(0, 1, 0.5), plt_hsl(120, 1, 0.5), 'multiply');
```

```sql {"type":"duckfn","show":"table"}
SELECT plt_to_hex(plt_blend(plt_convert(plt_hsl(0, 1, 0.5), 'srgb'),
                            plt_convert(plt_hsl(120, 1, 0.5), 'srgb'),
                            'multiply')) AS multiplied;
```

## `plt_blend(source, destination, mode)`

**Signature**: `plt_blend(source plt_color, destination plt_color, mode VARCHAR) -> plt_color`

The 11 separable modes, as in the CSS/PDF compositing model:

| Mode | Result in one line |
| --- | --- |
| `multiply` | darker: `source × destination` |
| `screen` | lighter: inverse of multiplying the inverses |
| `overlay` | multiply or screen, depending on the destination |
| `darken` | the darker of the two, per component |
| `lighten` | the lighter of the two, per component |
| `dodge` | brightens the destination towards white |
| `burn` | darkens the destination towards black |
| `hard_light` | overlay with source and destination swapped |
| `soft_light` | a softer hard light |
| `difference` | absolute difference |
| `exclusion` | difference with lower contrast |

```sql {"type":"duckfn","show":"table"}
SELECT plt_blend_modes() AS modes;
```

```sql {"type":"duckfn","show":"table"}
SELECT plt_to_hex(plt_blend(plt_srgb(0.5, 0.2, 0.1), plt_srgb(0.4, 0.6, 0.8), 'multiply')) AS multiply,
       plt_to_hex(plt_blend(plt_srgb(0.5, 0.2, 0.1), plt_srgb(0.4, 0.6, 0.8), 'screen')) AS screen,
       plt_to_hex(plt_blend(plt_srgb(0.5, 0.2, 0.1), plt_srgb(0.4, 0.6, 0.8), 'overlay')) AS overlay;
```

The alpha of the result follows from the blend: opaque source and destination give an opaque
result, and the destination shows through a translucent source:

```sql {"type":"duckfn","show":"table"}
SELECT plt_blend(plt_with_alpha(plt_srgb(0.5, 0.2, 0.1), 0.5),
                 plt_with_alpha(plt_srgb(0.4, 0.6, 0.8), 0.5),
                 'hard-light') AS result;
```

Mode names are normalized: case-insensitive, and `-` or a space counts as `_`, so
`'Hard Light'` is the same as `'hard_light'`. An unknown name fails the statement:

```sql {"type":"duckfn","expect":"error"}
SELECT plt_blend(plt_srgb(1, 0, 0), plt_srgb(0, 0, 1), 'glow');
```

## `plt_compose(source, destination, operator)`

**Signature**: `plt_compose(source plt_color, destination plt_color, operator VARCHAR) -> plt_color`

The six Porter-Duff operators:

| Operator | Keeps |
| --- | --- |
| `over` | source over destination (the usual alpha compositing) |
| `inside` | the part of the source inside the destination |
| `outside` | the part of the source outside the destination |
| `atop` | source over destination, limited to the destination's area |
| `xor` | the non-overlapping parts of both |
| `plus` | the sum of both, alpha included |

```sql {"type":"duckfn","show":"table"}
SELECT plt_compose_modes() AS operators;
```

```sql {"type":"duckfn","show":"table"}
SELECT plt_to_hex(plt_compose(plt_with_alpha(plt_srgb(1, 0, 0), 0.5),
                              plt_with_alpha(plt_srgb(0, 0, 1), 0.5), 'over')) AS over,
       plt_to_hex(plt_compose(plt_with_alpha(plt_srgb(1, 0, 0), 0.5),
                              plt_with_alpha(plt_srgb(0, 0, 1), 0.5), 'xor')) AS xor_,
       plt_compose(plt_with_alpha(plt_srgb(1, 0, 0), 0.5),
                   plt_with_alpha(plt_srgb(0, 0, 1), 0.5), 'plus') AS plus_alpha;
```

`plt_compose` also works in `lab`, `luv`, `yxy`, `oklab` and `cam16_ucs_jab`, which
`plt_blend` does not:

```sql {"type":"duckfn","show":"table"}
SELECT plt_compose(plt_with_alpha(plt_oklab(0.5, 0.1, 0.1), 0.5),
                   plt_with_alpha(plt_oklab(0.7, -0.1, 0.1), 0.5), 'over') AS in_oklab;
```

## `plt_blend_equations(source, destination, source_factor, destination_factor)`

**Signature**: `plt_blend_equations(source plt_color, destination plt_color, source_factor VARCHAR, destination_factor VARCHAR) -> plt_color`

The OpenGL-style form: you choose how much of the source and of the destination goes into the
result. Any pair of the ten factors is accepted:

```sql {"type":"duckfn","show":"table"}
SELECT plt_blend_parameters() AS factors;
```

```sql {"type":"duckfn","show":"table"}
SELECT plt_to_hex(plt_blend_equations(plt_with_alpha(plt_srgb(1, 0, 0), 0.5),
                                      plt_with_alpha(plt_srgb(0, 0, 1), 0.5),
                                      'source_alpha', 'one_minus_source_alpha')) AS blended;
```

`'source_alpha'` / `'one_minus_source_alpha'` reproduces ordinary `over` compositing, which is a
good way to check that a pair of factors does what you expect:

```sql {"type":"duckfn","show":"table"}
SELECT plt_to_hex(plt_compose(src, dst, 'over')) AS compose_over,
       plt_to_hex(plt_blend_equations(src, dst, 'source_alpha', 'one_minus_source_alpha')) AS equations
FROM (SELECT plt_with_alpha(plt_srgb(1, 0, 0), 0.5) AS src,
             plt_with_alpha(plt_srgb(0, 0, 1), 0.5) AS dst);
```

An unknown factor or operator is an error:

```sql {"type":"duckfn","expect":"error"}
SELECT plt_blend_equations(plt_srgb(1, 0, 0), plt_srgb(0, 0, 1), 'half', 'one');
```

```sql {"type":"duckfn","expect":"error"}
SELECT plt_compose(plt_srgb(1, 0, 0), plt_srgb(0, 0, 1), 'around');
```

## Listing the supported names

### `plt_blend_modes()`

**Signature**: `plt_blend_modes() -> VARCHAR[]`

The 11 mode names accepted by `plt_blend`, in a stable order. Use it to drive a
`list_transform` over a table of colors instead of hard-coding the list.

### `plt_compose_modes()`

**Signature**: `plt_compose_modes() -> VARCHAR[]`

The six operator names accepted by `plt_compose`.

### `plt_blend_parameters()`

**Signature**: `plt_blend_parameters() -> VARCHAR[]`

The ten factor names accepted by `plt_blend_equations`: `zero`, `one`, `source_color`,
`one_minus_source_color`, `destination_color`, `one_minus_destination_color`, `source_alpha`,
`one_minus_source_alpha`, `destination_alpha`, `one_minus_destination_alpha`.

```sql {"type":"duckfn","show":"table"}
SELECT unnest(plt_blend_modes()) AS mode, 'blend' AS used_by
FROM (SELECT 1)
UNION ALL
SELECT unnest(plt_compose_modes()), 'compose'
UNION ALL
SELECT unnest(plt_blend_parameters()), 'equations';
```
