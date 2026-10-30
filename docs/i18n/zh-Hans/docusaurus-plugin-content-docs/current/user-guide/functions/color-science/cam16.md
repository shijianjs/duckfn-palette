---
title: 自定义观察条件的 CAM16
sidebar_position: 2
description: 用命名或数值白点构造 CAM16 观察条件，并把 xyz 转换成完整的 CAM16 属性或八个 CAM16 空间中的任意一个。
---

# 自定义观察条件的 CAM16

`plt_convert` 能把颜色送进八个 `cam16_*` space 中的任意一个，但它始终使用 palette 的默认观察条件：
D65、适应亮度 40、背景亮度 0.2、平均环绕条件，以及自动折扣。对「就是转一下」来说，这是正确的选择。

这一页管的是另一种情况：你知道颜色是在什么条件下被观看的，并且希望这些条件参与计算。流程永远是
一样的：

1. 用 `plt_cam16_params` 或 `plt_cam16_params_xyz` 构造一个 `plt_cam16_params` 值；
2. 用这个值在 `xyz` 与 CAM16 之间转换。

这些函数只接受 `xyz`。如果你的颜色在别的 space，先转换过来，算完再转回去。

```sql {"type":"duckfn","show":"table"}
SELECT plt_cam16_from_xyz(plt_convert(plt_srgb(1, 0.5, 0), 'xyz'),
                          plt_cam16_default_params()) AS attributes;
```

## 构造观察条件

### `plt_cam16_default_params()`

**签名**：`plt_cam16_default_params() -> plt_cam16_params`

palette 的默认值，作为一个你可以传递下去的值。用它得到的结果与 `plt_convert(..., 'cam16_jch')`
完全一致：

```sql {"type":"duckfn","show":"table"}
SELECT plt_cam16_default_params() AS params;
```

### `plt_cam16_params(white_point, adapting_luminance, background_luminance, surround)` 与 `plt_cam16_params(white_point, adapting_luminance, background_luminance, surround, discounting)`

**签名**：`plt_cam16_params(white_point VARCHAR, adapting_luminance DOUBLE, background_luminance DOUBLE, surround VARCHAR) -> plt_cam16_params`
**签名**：`plt_cam16_params(white_point VARCHAR, adapting_luminance DOUBLE, background_luminance DOUBLE, surround VARCHAR, discounting DOUBLE) -> plt_cam16_params`

白点是 `plt_white_points()` 里的一个名字。两个亮度分别以 `cd/m²` 与 `0..1` 为单位（palette 的 `La`
与 `Yb`）。`surround` 是 `dark`、`dim`、`average`（对应 0%、10%、20% 视野），或者写成文本的百分比，
例如 `'12.5'`。

不带第五个参数时，折扣是**自动**的 —— palette 从亮度与环绕条件推导出来。带上它时，折扣就是给定的
0..1 因子：

```sql {"type":"duckfn","show":"table"}
SELECT plt_cam16_params('d65', 40, 0.2, 'average') AS defaults,
       plt_cam16_params('d50', 100, 0.2, 'dim') AS print_viewing,
       plt_cam16_params('d65', 40, 0.2, '12.5') AS custom_surround,
       plt_cam16_params('d65', 40, 0.2, 'dim', 0.9) AS fixed_discounting;
```

### `plt_cam16_params_xyz(white_x, white_y, white_z, adapting_luminance, background_luminance, surround)` 与 `plt_cam16_params_xyz(white_x, white_y, white_z, adapting_luminance, background_luminance, surround, discounting)`

**签名**：`plt_cam16_params_xyz(white_x DOUBLE, white_y DOUBLE, white_z DOUBLE, adapting_luminance DOUBLE, background_luminance DOUBLE, surround VARCHAR) -> plt_cam16_params`
**签名**：`plt_cam16_params_xyz(white_x DOUBLE, white_y DOUBLE, white_z DOUBLE, adapting_luminance DOUBLE, background_luminance DOUBLE, surround VARCHAR, discounting DOUBLE) -> plt_cam16_params`

同样的东西，只是白点用三个数字给出 —— palette 的动态白点。这是给没有名字的非标准光源用的：

```sql {"type":"duckfn","show":"table"}
SELECT plt_cam16_params_xyz(0.95047, 1, 1.08883, 40, 0.2, 'average') AS same_as_d65,
       plt_cam16_params_xyz(0.94, 1, 1.05, 40, 0.2, 'average', 0.8) AS custom_white;
```

### `plt_cam16_surrounds()`

**签名**：`plt_cam16_surrounds() -> VARCHAR[]`

三个预设名字及其百分比：`dark`（0%）、`dim`（10%）、`average`（20%）。凡是接受名字的地方，也接受
数值百分比。

```sql {"type":"duckfn","show":"table"}
SELECT plt_cam16_surrounds() AS surrounds;
```

## 转换

### `plt_cam16_from_xyz(color, params)`

**签名**：`plt_cam16_from_xyz(color plt_color, params plt_cam16_params) -> plt_cam16_full`

把一个 `xyz` 颜色转换成**完整**的 CAM16 属性集，即一个七字段的 struct：

```text
STRUCT(lightness, chroma, hue, brightness, colorfulness, saturation, alpha)
```

`J`、`C`、`h` 分别是明度、彩度与色相；`Q`、`M`、`s` 是亮度、色彩度与饱和度。alpha 从输入照抄。

```sql {"type":"duckfn","show":"table"}
SELECT plt_cam16_from_xyz(plt_xyz(0.4, 0.3, 0.2), plt_cam16_default_params()) AS attrs;
```

条件是起作用的：同一个 `xyz` 在不同条件下给出不同的属性。

```sql {"type":"duckfn","show":"table"}
SELECT (plt_cam16_from_xyz(c, plt_cam16_params('d65', 40, 0.2, 'average'))).lightness AS average,
       (plt_cam16_from_xyz(c, plt_cam16_params('d65', 40, 0.2, 'dark'))).lightness AS dark,
       (plt_cam16_from_xyz(c, plt_cam16_params('d65', 200, 0.2, 'average'))).lightness AS bright
FROM (SELECT plt_xyz(0.4, 0.3, 0.2) AS c);
```

### `plt_cam16_to_xyz(cam16, params)`

**签名**：`plt_cam16_to_xyz(cam16 plt_cam16_full, params plt_cam16_params) -> plt_color`

反向操作，使用同样的条件。palette 假定这些属性彼此一致，转回时只用到其中一部分；结果是一个 `xyz`
颜色，alpha 取自那个完整 struct。

```sql {"type":"duckfn","show":"table"}
SELECT plt_to_hex(plt_convert(plt_cam16_to_xyz(plt_cam16_from_xyz(plt_xyz(0.4, 0.3, 0.2),
                                                                 plt_cam16_default_params()),
                                               plt_cam16_default_params()), 'srgb')) AS round_trip;
```

### `plt_cam16_partial_from_xyz(color, space, params)`

**签名**：`plt_cam16_partial_from_xyz(color plt_color, space VARCHAR, params plt_cam16_params) -> plt_color`

把 `xyz` 颜色转换成八个 CAM16 space 之一，使用你的条件。结果是一个普通的 `plt_color`，它的
`space` 就是所请求的 CAM16 名字，所以还能用 `plt_convert` 继续转换（那时又会用回默认条件 —— 条件并不
携带在值里）：

```sql {"type":"duckfn","show":"table"}
SELECT plt_cam16_spaces() AS spaces;
```

```sql {"type":"duckfn","show":"table"}
SELECT plt_cam16_partial_from_xyz(plt_xyz(0.4, 0.3, 0.2), 'cam16_jch',
                                  plt_cam16_default_params()) AS jch,
       plt_cam16_partial_from_xyz(plt_xyz(0.4, 0.3, 0.2), 'cam16_ucs_jmh',
                                  plt_cam16_default_params()) AS ucs_jmh;
```

```sql {"type":"duckfn","show":"table"}
SELECT space,
       plt_components(plt_cam16_partial_from_xyz(plt_xyz(0.4, 0.3, 0.2), space,
                                                 plt_cam16_default_params())) AS components
FROM unnest(plt_cam16_spaces()) t(space);
```

### `plt_cam16_partial_to_xyz(color, params)`

**签名**：`plt_cam16_partial_to_xyz(color plt_color, params plt_cam16_params) -> plt_color`

把八个 CAM16 space 中的任意一个在给定条件下转回 `xyz`。颜色必须已经在某个 CAM16 space 里；没有隐式
转换，所以给错 space 会报错：

```sql {"type":"duckfn","show":"table"}
SELECT plt_components(plt_cam16_partial_to_xyz(
           plt_cam16_partial_from_xyz(plt_xyz(0.4, 0.3, 0.2), 'cam16_jmh',
                                      plt_cam16_default_params()),
           plt_cam16_default_params())) AS round_trip;
```

```sql {"type":"duckfn","expect":"error"}
SELECT plt_cam16_partial_to_xyz(plt_srgb(1, 0, 0), plt_cam16_default_params());
```

```sql {"type":"duckfn","expect":"error"}
SELECT plt_cam16_partial_from_xyz(plt_xyz(0.4, 0.3, 0.2), 'oklch', plt_cam16_default_params());
```

### `plt_cam16_spaces()`

**签名**：`plt_cam16_spaces() -> VARCHAR[]`

八个 CAM16 space 名字：`cam16_jch`、`cam16_jmh`、`cam16_jsh`、`cam16_qch`、`cam16_qmh`、
`cam16_qsh`、`cam16_ucs_jab`、`cam16_ucs_jmh`。

## 默认条件与自定义条件

| 任务 | 用什么 |
| --- | --- |
| 把颜色转成 CAM16 用于比较或存储 | `plt_convert(color, 'cam16_ucs_jab')` |
| 比较 CAM16 色差 | `plt_difference(a, b, 'delta_e', 'cam16_ucs_jab')` |
| 模拟某块屏幕、某个房间或某种光源 | 这一页的 `plt_cam16_*` 函数 |
| 一次查看全部六个属性 | `plt_cam16_from_xyz` |

前两条用 palette 的默认值，是省事的那条路。后两条才是需要把 `plt_cam16_params` 值传来传去的 ——
通常只构造一次，然后整张表复用：

```sql {"type":"duckfn","show":"table"}
SELECT plt_to_hex(plt_convert(plt_cam16_partial_to_xyz(
           plt_cam16_partial_from_xyz(plt_convert(c, 'xyz'), 'cam16_jch', p), p), 'srgb')) AS same_color
FROM (SELECT plt_srgb(0.2, 0.4, 0.6) AS c,
             plt_cam16_params('d65', 40, 0.2, 'average') AS p);
```
