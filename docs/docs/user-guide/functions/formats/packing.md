---
title: 8-bit packing
sidebar_position: 2
description: Quantize a color to 8-bit sRGB and pack it into a 32-bit integer in ARGB, RGBA, ABGR or BGRA order, and unpack it again.
---

# 8-bit packing

A packed color is a single integer — handy for storing a color in one `BIGINT` column, sending it
to a frontend, or matching a format another tool already uses. Packing converts to sRGB, clamps
into gamut, quantizes each channel and alpha to 8 bits, then puts the four bytes in the chosen
order. Unpacking reverses it and always returns an `srgb` color.

## `plt_pack(color)` and `plt_pack(color, order)`

**Signature**: `plt_pack(color plt_color) -> BIGINT`
**Signature**: `plt_pack(color plt_color, order VARCHAR) -> BIGINT`

The one-argument form uses `argb`, palette's default, which makes the low 24 bits read like a
`#rrggbb` value. The two-argument form takes `argb`, `rgba`, `abgr` or `bgra`.

```sql {"type":"duckfn","show":"table"}
SELECT plt_pack(plt_srgb(1, 0.5, 0)) AS argb,
       plt_pack(plt_srgb(1, 0.5, 0), 'rgba') AS rgba,
       plt_pack(plt_srgb(1, 0.5, 0), 'bgra') AS bgra;
```

```sql {"type":"duckfn","show":"table"}
SELECT plt_pack_orders() AS orders;
```

Alpha is part of the integer. An opaque color has `255` in the alpha byte, a half-transparent one
has `128`:

```sql {"type":"duckfn","show":"table"}
SELECT plt_pack(plt_color('srgb', 1, 0.5, 0, 1)) AS opaque,
       plt_pack(plt_color('srgb', 1, 0.5, 0, 0.5)) AS half_transparent;
```

Packing works from any space; the conversion to sRGB is implicit:

```sql {"type":"duckfn","show":"table"}
SELECT plt_pack(plt_oklch(0.7, 0.1, 120)) AS from_oklch,
       plt_pack(plt_lab(50, 200, 200)) AS out_of_gamut_clamped;
```

## `plt_unpack(value)` and `plt_unpack(value, order)`

**Signature**: `plt_unpack(value BIGINT) -> plt_color`
**Signature**: `plt_unpack(value BIGINT, order VARCHAR) -> plt_color`

The inverse. The order must match the one used to pack. Values outside `0..4294967295` are an
error; an unknown order is too.

```sql {"type":"duckfn","show":"table","colors":"round_trip"}
SELECT plt_to_hex(plt_unpack(plt_pack(plt_srgb(1, 0.5, 0)))) AS round_trip,
       plt_unpack(plt_pack(plt_srgb(1, 0.5, 0))) AS unpacked;
```

```sql {"type":"duckfn","show":"table","colors":["rgba_round_trip","with_alpha"]}
SELECT plt_to_hex(plt_unpack(plt_pack(plt_srgb(1, 0.5, 0), 'rgba'), 'rgba')) AS rgba_round_trip,
       plt_to_hex(plt_unpack(plt_pack(plt_color('srgb', 1, 0.5, 0, 0.5), 'bgra'), 'bgra')) AS with_alpha;
```

The round trip is lossy by design: components come back as multiples of `1/255`. Use it for
storage and transport, not as an alternative to `plt_convert`.

```sql {"type":"duckfn","show":"table"}
SELECT plt_components(plt_srgb(0.2, 0.4, 0.6)) AS original,
       plt_components(plt_unpack(plt_pack(plt_srgb(0.2, 0.4, 0.6)))) AS after_round_trip;
```

```sql {"type":"duckfn","expect":"error"}
SELECT plt_unpack(4294967296);
```

```sql {"type":"duckfn","expect":"error"}
SELECT plt_unpack(0, 'rgb');
```

## `plt_pack_orders()`

**Signature**: `plt_pack_orders() -> VARCHAR[]`

The four channel orders: `argb`, `rgba`, `abgr`, `bgra`. Names are case-insensitive, and `-` or a
space counts as `_`.
