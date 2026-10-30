---
title: 随机色
sidebar_position: 2
description: 在某个 color space 里采样可复现的随机色，一次一个，或作为带 seed 的序列。
---

# 随机色

这些函数从 palette 的分布里采样：「在 space 内均匀」，具体怎么均匀由 palette 为每个 space 定义
（sRGB 各分量在 0..1 上，Lab 的 `a`/`b` 在 -128..127 上，色相绕整圈，饱和度/亮度按该 space 色体
的形状加权，等等）。

每个采样函数都要一个 **seed**。相同的 space 与 seed 永远给出相同的颜色，所以结果在不同查询、不同
机器之间都可复现，放在 `EXPLAIN` 里也是安全的。

## `plt_random_color(space, seed)`

**签名**：`plt_random_color(space VARCHAR, seed BIGINT) -> plt_color`

在给定 space 里的一个颜色。

```sql {"type":"duckfn","show":"table"}
SELECT plt_random_color('oklch', 42) AS color,
       plt_to_hex(plt_random_color('oklch', 42)) AS as_hex;
```

seed 就是全部状态：换一个得到不同的颜色，保持不变就得到同一个。

```sql {"type":"duckfn","show":"table"}
SELECT seed, plt_to_hex(plt_random_color('srgb', seed)) AS color
FROM range(1, 6) t(seed);
```

## `plt_random_colors(space, count, seed)`

**签名**：`plt_random_colors(space VARCHAR, count BIGINT, seed BIGINT) -> plt_color[]`

从同一条带 seed 的序列里取出的 `count` 个颜色。逐次取出的值与相同 seed 的 `plt_random_color` 不同，
但对给定的 `(space, count, seed)`，整条序列是固定的。

```sql {"type":"duckfn","show":"table"}
SELECT to_json(list_transform(plt_random_colors('srgb', 5, 42), lambda x: plt_to_hex(x))) AS colors,
       length(plt_random_colors('srgb', 5, 42)) AS count;
```

count 为 0 得到空列表；负数会报错：

```sql {"type":"duckfn","show":"table"}
SELECT length(plt_random_colors('lab', 0, 7)) AS empty;
```

```sql {"type":"duckfn","expect":"error"}
SELECT plt_random_colors('srgb', -1, 42);
```

生成测试数据或随机分类色板时很好用：

```sql {"type":"duckfn","show":"table"}
SELECT i, plt_to_hex(colors[i]) AS color
FROM (SELECT plt_random_colors('srgb', 6, 2026) AS colors) t,
     range(1, 7) r(i);
```

## `plt_random_spaces()`

**签名**：`plt_random_spaces() -> VARCHAR[]`

有随机分布的 34 个 space：15 个 RGB 空间（gamma 与线性两种变体）、`hsl`、`hsv`、`hwb`、`lab`、
`lch`、`luv`、`lchuv`、`xyz`、`xyz_d50`、`yxy`、`luma`、`luma_linear`、`luma_gamma`、`oklab`、
`oklch`、`cam16_ucs_jab`、`cam16_ucs_jmh`、`lms` 与 `lms_von_kries`。

```sql {"type":"duckfn","show":"table"}
SELECT list_count(plt_random_spaces()) AS count,
       plt_random_spaces() AS spaces;
```

不在这个列表里的 space —— `hsluv`、`okhsl`、`okhsv`、`okhwb`，以及部分的 CAM16 空间
（`cam16_jch`、`cam16_jmh`、`cam16_jsh`、`cam16_qch`、`cam16_qmh`、`cam16_qsh`）—— 没有 palette
分布，无法采样：

```sql {"type":"duckfn","expect":"error"}
SELECT plt_random_color('hsluv', 42);
```

```sql {"type":"duckfn","expect":"error"}
SELECT plt_random_color('oklchh', 42);
```

在列表里的 space 中采样，需要别的 space 时之后再转换：

```sql {"type":"duckfn","show":"table"}
SELECT plt_convert(plt_random_color('oklch', 7), 'lch') AS as_lch;
```

## 错误

- 未知的 space 名，或者没有随机分布的 space，会让查询失败，错误信息会写明是哪个函数并列出支持的
  space。
- `count` 为负数会让查询失败。
- 任一参数为 NULL 返回 NULL。
