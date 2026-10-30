---
title: 色差与对比度
sidebar_position: 3
description: 七种色差指标及其有定义的空间，外加 WCAG 2.1 的相对亮度、对比度与达标判定。
---

# 色差与对比度

两族函数回答的是不同问题。**色差**是一个数字，回答「这两个颜色看起来差多远」，用哪种指标就决定了在
哪个 space 里度量。**WCAG 对比度**是文本与图形的规范比值，始终在 sRGB 里计算。

## `plt_difference(a, b, metric)` 与 `plt_difference(a, b, metric, space)`

**签名**：`plt_difference(a plt_color, b plt_color, metric VARCHAR) -> DOUBLE`
**签名**：`plt_difference(a plt_color, b plt_color, metric VARCHAR, space VARCHAR) -> DOUBLE`

三参数形式在 CIE L\*a\*b\*（D65）里度量 —— ΔE、HyAb 与 CIEDE2000 就定义在那里。四参数形式把**两个**
颜色都转换到 `space`（不做夹紧）再在那儿度量；如果该指标在这个 space 里没有定义，语句会失败，错误
信息里会列出可用的 space。

```sql {"type":"duckfn","show":"table"}
SELECT plt_difference(plt_lab(50, 2.6772, -79.7751), plt_lab(50, 0, -82.7485), 'ciede2000') AS delta_e,
       plt_difference(plt_lab(50, 2.6772, -79.7751), plt_lab(50, 0, -82.7485), 'ciede2000', 'lch') AS same_in_lch;
```

| 指标 | 含义 | 有定义的空间 |
| --- | --- | --- |
| `euclidean` | 在该 space 各分量上的直线距离 | `cam16_ucs_jab`、`lab`、`lms`、`lms_von_kries`、`luma`、`luma_linear`、`luma_gamma`、`luv`、`oklab`、每个 RGB space、`xyz`、`xyz_d50`、`yxy` |
| `euclidean_squared` | 同上，取平方（排序更省，次序不变） | 与 `euclidean` 相同 |
| `delta_e` | CIE76 ΔE\*ab | `cam16_ucs_jab`、`cam16_ucs_jmh`、`lab`、`lch` |
| `improved_delta_e` | 加上 palette 修正的 ΔE\*ab | `cam16_ucs_jab`、`cam16_ucs_jmh`、`lab`、`lch` |
| `hyab` | 混合距离：Lab 亮度差加上 a/b（或 u/v）距离 | `cam16_ucs_jab`、`lab`、`luv`、`oklab` |
| `ciede2000` | CIEDE2000 ΔE\*00，标准的感知指标 | `lab`、`lch` |
| `improved_ciede2000` | 修掉不连续处的 CIEDE2000 | `lab`、`lch` |

别名：`cie76`、`deltae` 与 `delta_e_76` 都指 `delta_e`；`cie2000` 与 `delta_e_2000` 都指
`ciede2000`。名字大小写不敏感，`-` 与空格视作 `_`。

```sql {"type":"duckfn","show":"table"}
SELECT plt_difference_metrics() AS metrics;
```

```sql {"type":"duckfn","show":"table"}
SELECT plt_difference(plt_srgb(1, 0, 0), plt_srgb(0, 1, 0), 'euclidean', 'oklab') AS oklab_distance,
       plt_difference(plt_srgb(1, 0, 0), plt_srgb(0, 1, 0), 'euclidean', 'srgb') AS srgb_distance,
       plt_difference(plt_srgb(1, 0, 0), plt_srgb(1, 0.05, 0), 'delta_e') AS nearly_identical;
```

指标用在错误的 space 里会报错，而不是悄悄做一次转换：

```sql {"type":"duckfn","expect":"error"}
SELECT plt_difference(plt_srgb(1, 0, 0), plt_srgb(0, 1, 0), 'ciede2000', 'srgb');
```

```sql {"type":"duckfn","expect":"error"}
SELECT plt_difference(plt_srgb(1, 0, 0), plt_srgb(0, 1, 0), 'lab-distance');
```

## `plt_relative_luminance(color)`

**签名**：`plt_relative_luminance(color plt_color) -> DOUBLE`

WCAG 2.1 的相对亮度：黑色为 0，白色为 1。颜色会先转换到 sRGB，因为这个定义以 sRGB 为前提。对比度
正是由它算出来的；单独用它也能判断一个颜色算「亮」还是「暗」：

```sql {"type":"duckfn","show":"table"}
SELECT plt_relative_luminance(plt_srgb(0, 0, 0)) AS black,
       plt_relative_luminance(plt_srgb(0.5, 0.5, 0.5)) AS middle_gray,
       plt_relative_luminance(plt_srgb(1, 1, 1)) AS white,
       plt_relative_luminance(plt_lab(50, 0, 0)) AS lab_gray;
```

## `plt_contrast(a, b)`

**签名**：`plt_contrast(a plt_color, b plt_color) -> DOUBLE`

WCAG 2.1 的对比度 `(L_lighter + 0.05) / (L_darker + 0.05)`，从 1（完全相同）到 21（黑对白）。两个
颜色都会转换到 sRGB。结果是对称的：

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

**签名**：`plt_contrast_check(a plt_color, b plt_color, criterion VARCHAR) -> BOOLEAN`

用同一个比值直接回答某条 WCAG 准则：

| 准则 | 最低比值 |
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

一个常用做法是给一套色板逐色标注，再按它必须满足的准则过滤：

```sql {"type":"duckfn","show":"table"}
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

## 列出支持的名字

### `plt_difference_metrics()`

**签名**：`plt_difference_metrics() -> VARCHAR[]`

`plt_difference` 接受的七个指标名。

### `plt_contrast_criteria()`

**签名**：`plt_contrast_criteria() -> VARCHAR[]`

`plt_contrast_check` 接受的五个准则名。
