---
title: 配色方案
sidebar_position: 4
description: 互补、分裂互补、邻近、三角与四角配色方案，通过在颜色自身的 space 里旋转色相算出，外加 tinycolor 那样的单色方案。
---

# 配色方案

配色方案是从一个颜色出发、旋转它的色相得到的一组颜色。这些函数都在**颜色自身的 space 里**工作：
`oklch` 输入给出 `oklch` 结果，亮度与 chroma 不变、色相被旋转并归一化到 0..360。alpha 会保留。

它们需要带色相的 space。支持的有：`hsl`、`hsv`、`hwb`、`hsluv`、`lch`、`lchuv`、`oklch`、`okhsl`、
`okhsv`、`okhwb`，六个部分 CAM16 space `cam16_jch`、`cam16_jmh`、`cam16_jsh`、`cam16_qch`、
`cam16_qmh`、`cam16_qsh`，以及 `cam16_ucs_jmh`。没有色相的 space 会被拒绝：

```sql {"type":"duckfn","expect":"error"}
SELECT plt_complementary(plt_lab(50, 20, -30));
```

先转换就是全部的修法：

```sql {"type":"duckfn","show":"table"}
SELECT plt_to_hex(plt_complementary(plt_convert(plt_srgb(1, 0, 0), 'oklch'))) AS complement;
```

这些函数返回方案里**其余**的颜色，顺序与 palette 一致，不包括输入颜色本身。它们共用同一种列表形状，
所以查询里可以一视同仁地处理：

```sql {"type":"duckfn","show":"table"}
SELECT to_json(list_transform(plt_triadic(plt_oklch(0.6, 0.15, 30)), lambda x: plt_to_hex(x))) AS triadic;
```

## `plt_complementary(color)`

**签名**：`plt_complementary(color plt_color) -> plt_color`

色相环上正对面的颜色：色相 + 180°。这个函数返回单个颜色而非列表，因为补色只有一个颜色。

```sql {"type":"duckfn","show":"table"}
SELECT plt_to_hex(plt_complementary(plt_hsl(120, 1, 0.5))) AS complement,
       plt_components(plt_complementary(plt_hsl(120, 1, 0.5))) AS complement_hsl;
```

## `plt_split_complementary(color)`

**签名**：`plt_split_complementary(color plt_color) -> plt_color[]`

色相 + 150° 与 + 210° 处的两个颜色 —— 补色的两个邻居。

```sql {"type":"duckfn","show":"table"}
SELECT plt_components(plt_split_complementary(plt_hsl(120, 1, 0.5))[1]) AS first,
       plt_components(plt_split_complementary(plt_hsl(120, 1, 0.5))[2]) AS second;
```

## `plt_analogous(color)`

**签名**：`plt_analogous(color plt_color) -> plt_color[]`

色相 − 30° 与 + 30° 处的两个颜色。

```sql {"type":"duckfn","show":"table"}
SELECT to_json(list_transform(plt_analogous(plt_hsl(120, 1, 0.5)), lambda x: plt_to_hex(x))) AS analogous;
```

## `plt_analogous_secondary(color)`

**签名**：`plt_analogous_secondary(color plt_color) -> plt_color[]`

再往外的一对：色相 − 60° 与 + 60°。它与 `plt_analogous` 合起来构成五色的邻近配色方案。

```sql {"type":"duckfn","show":"table"}
SELECT plt_components(plt_analogous_secondary(plt_hsl(120, 1, 0.5))[1]) AS minus_60,
       plt_components(plt_analogous_secondary(plt_hsl(120, 1, 0.5))[2]) AS plus_60;
```

## `plt_triadic(color)`

**签名**：`plt_triadic(color plt_color) -> plt_color[]`

色相 + 120° 与 + 240° 处的两个颜色。

```sql {"type":"duckfn","show":"table"}
SELECT to_json(list_transform(plt_triadic(plt_hsl(120, 1, 0.5)), lambda x: plt_to_hex(x))) AS triadic;
```

## `plt_tetradic(color)`

**签名**：`plt_tetradic(color plt_color) -> plt_color[]`

色相 + 90°、+ 180° 与 + 270° 处的三个颜色 —— 也就是正方形方案。

```sql {"type":"duckfn","show":"table"}
SELECT to_json(list_transform(plt_tetradic(plt_hsl(30, 1, 0.5)), lambda x: plt_to_hex(x))) AS tetradic;
```

## `plt_monochromatic(color, count)`

**签名**：`plt_monochromatic(color plt_color, count INTEGER) -> plt_color[]`

这里的例外：它保持色相与饱和度不变，让 HSV 的 **value** 从当前值起按 `1/count` 循环，并回绕到
0..1 —— 即 tinycolor 的 `monochromatic`。它恰好返回 `count` 个颜色，且第一个就是输入颜色本身，这一点
与上面的色相方案不同（那些不含输入色）。`count` 为 0 得到空列表，负数报错。结果留在输入的 space 里，
alpha 不变。

```sql {"type":"duckfn","show":"table"}
SELECT to_json(list_transform(plt_monochromatic(plt_srgb(1, 0, 0), 4),
                              lambda x: plt_to_hex(x))) AS reds,
       plt_components(plt_monochromatic(plt_hsl(120, 1, 0.5), 3)[2]) AS second_hsl;
```

## `plt_harmony(color, scheme)`

**签名**：`plt_harmony(color plt_color, scheme VARCHAR) -> plt_color[]`

六种方案按名字取用的入口。它接受 `plt_color_schemes()` 里的名字，外加别名 `complement`、
`split_complement`、`triad` 与 `square`。方案名来自某一列或某个参数时，就用它。六种方案都是旋转色相，
所以 `monochromatic` 不在其中 —— 它变的是 HSV 的 value，有自己的函数（见上）。

```sql {"type":"duckfn","show":"table"}
SELECT scheme,
       to_json(list_transform(plt_harmony(plt_oklch(0.6, 0.15, 30), scheme),
                              lambda x: plt_to_hex(x))) AS colors
FROM (VALUES ('complementary'), ('split_complementary'), ('analogous'),
             ('analogous_secondary'), ('triadic'), ('tetradic')) t(scheme);
```

```sql {"type":"duckfn","show":"table"}
SELECT plt_to_hex(plt_harmony(plt_oklch(0.6, 0.15, 30), 'Square')[1]) AS first_of_square;
```

```sql {"type":"duckfn","expect":"error"}
SELECT plt_harmony(plt_oklch(0.6, 0.15, 30), 'bogus');
```

## `plt_color_schemes()`

**签名**：`plt_color_schemes() -> VARCHAR[]`

六个规范的方案名：`complementary`、`split_complementary`、`analogous`、`analogous_secondary`、
`triadic`、`tetradic`。

```sql {"type":"duckfn","show":"table"}
SELECT plt_color_schemes() AS schemes;
```
