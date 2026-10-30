---
title: 8 位打包
sidebar_position: 2
description: 把颜色量化到 8 位 sRGB，按 ARGB、RGBA、ABGR 或 BGRA 顺序打包进一个 32 位整数，再解包回来。
---

# 8 位打包

打包后的颜色是一个整数 —— 方便把颜色存进单个 `BIGINT` 列、发给前端，或者对上另一个工具已经在用的
格式。打包会先转成 sRGB、夹进色域、把每个通道与 alpha 量化到 8 位，然后按选定的顺序摆好这四个字节。
解包把它倒过来，并且总是返回一个 `srgb` 颜色。

## `plt_pack(color)` 与 `plt_pack(color, order)`

**签名**：`plt_pack(color plt_color) -> BIGINT`
**签名**：`plt_pack(color plt_color, order VARCHAR) -> BIGINT`

单参数形式用 `argb`，也就是 palette 的默认顺序，这样低 24 位读起来就像一个 `#rrggbb` 值。双参数形式
接受 `argb`、`rgba`、`abgr` 或 `bgra`。

```sql {"type":"duckfn","show":"table"}
SELECT plt_pack(plt_srgb(1, 0.5, 0)) AS argb,
       plt_pack(plt_srgb(1, 0.5, 0), 'rgba') AS rgba,
       plt_pack(plt_srgb(1, 0.5, 0), 'bgra') AS bgra;
```

```sql {"type":"duckfn","show":"table"}
SELECT plt_pack_orders() AS orders;
```

alpha 是整数的一部分。不透明的颜色在 alpha 字节上是 `255`，半透明的颜色是 `128`：

```sql {"type":"duckfn","show":"table"}
SELECT plt_pack(plt_color('srgb', 1, 0.5, 0, 1)) AS opaque,
       plt_pack(plt_color('srgb', 1, 0.5, 0, 0.5)) AS half_transparent;
```

任何 space 都能打包；转成 sRGB 是隐含的：

```sql {"type":"duckfn","show":"table"}
SELECT plt_pack(plt_oklch(0.7, 0.1, 120)) AS from_oklch,
       plt_pack(plt_lab(50, 200, 200)) AS out_of_gamut_clamped;
```

## `plt_unpack(value)` 与 `plt_unpack(value, order)`

**签名**：`plt_unpack(value BIGINT) -> plt_color`
**签名**：`plt_unpack(value BIGINT, order VARCHAR) -> plt_color`

反向操作。顺序必须与打包时用的一致。`0..4294967295` 之外的值会报错，未知的顺序也一样。

```sql {"type":"duckfn","show":"table"}
SELECT plt_to_hex(plt_unpack(plt_pack(plt_srgb(1, 0.5, 0)))) AS round_trip,
       plt_unpack(plt_pack(plt_srgb(1, 0.5, 0))) AS unpacked;
```

```sql {"type":"duckfn","show":"table"}
SELECT plt_to_hex(plt_unpack(plt_pack(plt_srgb(1, 0.5, 0), 'rgba'), 'rgba')) AS rgba_round_trip,
       plt_to_hex(plt_unpack(plt_pack(plt_color('srgb', 1, 0.5, 0, 0.5), 'bgra'), 'bgra')) AS with_alpha;
```

往返是有意有损的：分量回来时是 `1/255` 的倍数。用它来存储与传输，不要把它当作 `plt_convert`
的替代品。

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

**签名**：`plt_pack_orders() -> VARCHAR[]`

四种通道顺序：`argb`、`rgba`、`abgr`、`bgra`。名字不区分大小写，`-` 或空格都当作 `_`。
