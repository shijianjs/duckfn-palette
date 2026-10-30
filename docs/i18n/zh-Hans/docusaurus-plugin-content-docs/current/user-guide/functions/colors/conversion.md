---
title: 转换与夹紧
sidebar_position: 2
description: 在 44 个空间中的任意两个之间移动颜色，夹紧或不夹紧，并检查颜色是否落在所属空间的范围内。
---

# 转换与夹紧

转换始终让空间标签与 alpha 保持一致：`plt_convert(c, 'lab')` 之后值里的空间是 `lab`，
`plt_convert(..., 'srgb')` 又能把它带回来。数学由 palette 完成；这个扩展只负责选路线。

## `plt_convert(color, space)`

**签名**：`plt_convert(color plt_color, space VARCHAR) -> plt_color`

转换到目标空间，**不做色域夹紧**。超出目标范围的值原样返回，这对检查中间结果、以及那些本应可逆的
转换很有用。

```sql {"type":"duckfn","show":"table"}
SELECT plt_to_hex(plt_srgb(1, 0.5, 0)) AS srgb,
       plt_components(plt_convert(plt_srgb(1, 0.5, 0), 'oklch')) AS as_oklch,
       plt_components(plt_convert(plt_srgb(1, 0.5, 0), 'xyz')) AS as_xyz;
```

alpha 是复制过去的，不参与转换：

```sql {"type":"duckfn","show":"table"}
SELECT plt_convert(plt_with_alpha(plt_srgb(1, 0, 0), 0.25), 'lab') AS converted,
       plt_alpha(plt_convert(plt_with_alpha(plt_srgb(1, 0, 0), 0.25), 'lab')) AS alpha;
```

## `plt_convert_clamped(color, space)`

**签名**：`plt_convert_clamped(color plt_color, space VARCHAR) -> plt_color`

先转换，再把结果夹进**目标**空间的标准范围。渲染之前你要的就是它：sRGB 之外的颜色会被拉回最接近的
色域内 sRGB 颜色，而不是得到 `1.5` 或 `-0.3` 这样的分量。

```sql {"type":"duckfn","show":"table"}
SELECT plt_components(plt_convert(plt_lab(50, 200, 200), 'srgb')) AS unclamped,
       plt_components(plt_convert_clamped(plt_lab(50, 200, 200), 'srgb')) AS clamped;
```

## `plt_clamp(color)`

**签名**：`plt_clamp(color plt_color) -> plt_color`

在颜色**自身**的空间里夹紧，不做转换。Lab 的 a\* 与 b\* 停在 ±127，RGB 分量停在 0..1，但色相不会被
这个函数强行限制在 0..360。

```sql {"type":"duckfn","show":"table"}
SELECT plt_components(plt_lab(50, 200, 200)) AS out_of_range,
       plt_components(plt_clamp(plt_lab(50, 200, 200))) AS clamped,
       plt_components(plt_srgb(-0.2, 1.4, 0.5)) AS out_of_gamut,
       plt_components(plt_clamp(plt_srgb(-0.2, 1.4, 0.5))) AS in_gamut;
```

## `plt_is_within_bounds(color)`

**签名**：`plt_is_within_bounds(color plt_color) -> BOOLEAN`

`plt_clamp` 是否会改变任何东西。把它和 `WHERE` 子句结合起来，就能找出渲染器不得不裁剪的颜色。

```sql {"type":"duckfn","show":"table"}
SELECT plt_is_within_bounds(plt_lab(50, 200, 200)) AS lab_ok,
       plt_is_within_bounds(plt_clamp(plt_lab(50, 200, 200))) AS clamped_ok,
       plt_is_within_bounds(plt_srgb(1, 0, 0)) AS srgb_ok;
```

```sql {"type":"duckfn","show":"table"}
SELECT c, plt_is_within_bounds(c) AS in_gamut
FROM (VALUES
        (plt_srgb(0.2, 0.4, 0.6)),
        (plt_srgb(-0.1, 0.5, 0.5)),
        (plt_lch(50, 120, 40))
     ) t(c);
```

## 一次转换走哪条路线？

所有转换都经过 XYZ (D65) —— palette 转换图的中心节点。编码的 RGB 空间会先被解码；D50 空间
（ProPhoto RGB、`xyz_d50`）用 Bradford 做色适应；CAM16 使用 palette 的默认观察条件。路线不由你选择，
但 `plt_convert` 与 `plt_convert_clamped` 在它上面算出的数字相同，而且经过任何可逆空间往返一次，
都能把原色还原到浮点精度：

```sql {"type":"duckfn","show":"table"}
SELECT plt_to_hex(plt_convert(plt_convert(plt_srgb(0.2, 0.4, 0.6), 'hsluv'), 'srgb')) AS round_trip;
```

一维的 `luma*` 空间按设计不可逆，而 `srgb_gamma` 是 palette 已废弃的伽马 2.2 编码 —— 除非你确实需要
它，否则用 `plt_srgb`。

## `plt_equals(a, b)`

**签名**：`plt_equals(a plt_color, b plt_color) -> BOOLEAN`

两个颜色渲染出来是否相同：都先转成 sRGB、每个通道量化到 8 位，alpha 保留两位小数。这就是 tinycolor
的 `equals`，比较来自不同 space 的颜色时通常正是你想要的结果 —— `plt_srgb(1, 0, 0)` 与
`plt_hsl(0, 1, 0.5)` 相等。感知均匀空间里可能差一丝的颜色，在这里也算相等：

```sql {"type":"duckfn","show":"table"}
SELECT plt_equals(plt_srgb(1, 0, 0), plt_hsl(0, 1, 0.5)) AS across_spaces,
       plt_equals(plt_with_alpha(plt_srgb(1, 0, 0), 0.5), plt_with_alpha(plt_srgb(1, 0, 0), 0.6)) AS alpha_differs,
       plt_equals(plt_srgb(0.5, 0.5, 0.5), plt_srgb(0.501, 0.5, 0.5)) AS within_a_byte;
```

直接用 `=` 比较两个 STRUCT 并不会这样：它会连 `space` 标签和完整的浮点分量一起比较，所以两个渲染
一致、只在最后一位上不同的 `srgb` 值在那里是 `!=`。

## 错误

未知的空间名会让语句失败；错误信息里带函数名：

```sql {"type":"duckfn","expect":"error"}
SELECT plt_convert(plt_srgb(1, 0, 0), 'nope');
```

```sql {"type":"duckfn","expect":"error"}
SELECT plt_convert_clamped(plt_srgb(1, 0, 0), 'not-a-space');
```
