---
title: 白点、色适应与编码
sidebar_position: 1
description: 命名白点、色适应矩阵、传递函数、RGB 三原色、RGB 转 XYZ 矩阵，以及通用的 3x3 矩阵辅助函数。
---

# 白点、色适应与编码

`plt_convert` 内部已经把这一切都处理好了 —— 它用 Bradford 把 D50 的 space 适应到 D65，并在途中解码
传递函数。这一页的函数把同样的零件暴露出来，好让你检视它们、手工复现一次转换，或者把它们组合成
`plt_convert` 做不到的东西。

这里没有任何函数会改变颜色的 space。唯一返回颜色的函数 `plt_chromatic_adapt` 保留 `xyz` 标签，只改
里面的数值。

## 白点

### `plt_white_points()`

**签名**：`plt_white_points() -> VARCHAR[]`

全部白点名字：CIE 光源 `a`、`b`、`c`、`d50`、`d55`、`d65`、`d75`、`e`、`f2`、`f7`、`f11`，它们的
10° 观察者变体 `d50_degree10`、`d55_degree10`、`d65_degree10`、`d75_degree10`，以及 `dci_p3`
（影院白）。

```sql {"type":"duckfn","show":"table"}
SELECT plt_white_points() AS white_points;
```

### `plt_white_point_xyz(name)`

**签名**：`plt_white_point_xyz(name VARCHAR) -> DOUBLE[]`

三刺激值 `[X, Y, Z]`，已归一化到 `Y = 1`。这就是你从 D65 认出来的那组 `[0.95047, 1, 1.08883]`：

```sql {"type":"duckfn","show":"table"}
SELECT plt_white_point_xyz('d65') AS d65,
       plt_white_point_xyz('d50') AS d50,
       plt_white_point_xyz('a') AS illuminant_a;
```

### `plt_white_point_xy(name)`

**签名**：`plt_white_point_xy(name VARCHAR) -> DOUBLE[]`

换成色度坐标 `[x, y]`，即它在 CIE 1931 图上的位置：

```sql {"type":"duckfn","show":"table"}
SELECT plt_white_point_xy('d65') AS d65,
       plt_white_point_xy('d50') AS d50;
```

名字会被规范化（`D65`、`d-65`、`d 65` 都能用），另外接受几个别名：`illuminant_a` …
`illuminant_e`，以及 10° 变体的 `d65_10` / `d65_10deg` / `d65_degree_10`。

```sql {"type":"duckfn","expect":"error"}
SELECT plt_white_point_xyz('d42');
```

## 色适应

### `plt_adaptation_methods()`

**签名**：`plt_adaptation_methods() -> VARCHAR[]`

三种方法：`bradford`（常用的默认值，也是 `plt_convert` 用的）、`von_kries` 与 `unit_matrix`（纯粹的
XYZ 缩放，适合当基线）。别名如 `vonkries`、`hpe`、`hunt_pointer_estevez`、`identity`、`none` 与
`xyz_scaling` 也被接受。

```sql {"type":"duckfn","show":"table"}
SELECT plt_adaptation_methods() AS methods;
```

### `plt_adaptation_matrix(source, target, method)`

**签名**：`plt_adaptation_matrix(source VARCHAR, target VARCHAR, method VARCHAR) -> DOUBLE[]`

把 `source` 下的 XYZ 值送到 `target` 下的 3×3 矩阵，行优先：

```sql {"type":"duckfn","show":"table"}
SELECT plt_adaptation_matrix('d65', 'd50', 'bradford') AS d65_to_d50,
       plt_adaptation_matrix('a', 'c', 'bradford') AS a_to_c;
```

这个矩阵由三块拼成，每一块都单独暴露：`plt_lms_matrix(method)` → 增益 →
`plt_lms_to_xyz_matrix(method)`。因此可以搭一条自己的链条：

```sql {"type":"duckfn","show":"table"}
SELECT plt_lms_matrix('bradford') AS xyz_to_lms,
       plt_diagonal_adaptation_matrix('d65', 'd50', 'bradford') AS gains,
       plt_lms_to_xyz_matrix('bradford') AS lms_to_xyz;
```

### `plt_lms_matrix(method)`

**签名**：`plt_lms_matrix(method VARCHAR) -> DOUBLE[]`

该方法对应的 XYZ→LMS 矩阵（Bradford、von Kries / Hunt-Pointer-Estevez，或基于单位矩阵的 unit
matrix）。

### `plt_lms_to_xyz_matrix(method)`

**签名**：`plt_lms_to_xyz_matrix(method VARCHAR) -> DOUBLE[]`

该方法对应的 LMS→XYZ 矩阵，即 `plt_lms_matrix` 的逆矩阵。

### `plt_diagonal_adaptation_matrix(source, target, method)`

**签名**：`plt_diagonal_adaptation_matrix(source VARCHAR, target VARCHAR, method VARCHAR) -> DOUBLE[]`

两个白点之间的对角 LMS 增益矩阵。它只在两个 LMS 矩阵**之间**才有意义：

```sql {"type":"duckfn","show":"table"}
SELECT plt_diagonal_adaptation_matrix('d65', 'd50', 'bradford') AS gains;
```

### `plt_chromatic_adapt(color, target, method)` 与 `plt_chromatic_adapt(color, source, target, method)`

**签名**：`plt_chromatic_adapt(color plt_color, target VARCHAR, method VARCHAR) -> plt_color`
**签名**：`plt_chromatic_adapt(color plt_color, source VARCHAR, target VARCHAR, method VARCHAR) -> plt_color`

把适应矩阵作用到一个颜色上。颜色**必须处在 `xyz`**，因为它的分量就是矩阵所作用的三刺激值。三参数
形式假定分量处在 D65 下；四参数形式显式给出源白点。

返回值的标签是 `xyz`，携带的是目标白点下的数值。白点本身并不是值的一部分，所以你得自己记住它 ——
`plt_xyz` 意味着 D65，而一个适应到 `d50` 的结果是同一个标签、不同的数值。

```sql {"type":"duckfn","show":"table"}
SELECT plt_chromatic_adapt(plt_xyz(0.5, 0.4, 0.3), 'd50', 'bradford') AS adapted,
       plt_chromatic_adapt(plt_xyz(0.5, 0.4, 0.3), 'd65', 'd50', 'bradford') AS explicit_source;
```

```sql {"type":"duckfn","show":"table"}
SELECT plt_components(c) AS original,
       plt_components(plt_chromatic_adapt(c, 'a', 'bradford')) AS under_a,
       plt_components(c) AS still_original
FROM (SELECT plt_xyz(0.5, 0.4, 0.3) AS c);
```

用矩阵与 `plt_apply_matrix` 手工做同样的适应，得到的是同样的数值：

```sql {"type":"duckfn","show":"table"}
SELECT plt_components(plt_chromatic_adapt(c, 'd50', 'bradford')) AS adapted,
       plt_components(plt_apply_matrix(plt_adaptation_matrix('d65', 'd50', 'bradford'), c)) AS by_hand
FROM (SELECT plt_xyz(0.5, 0.4, 0.3) AS c);
```

处在别的 space 的颜色会报错，而不是被隐式转换：

```sql {"type":"duckfn","expect":"error"}
SELECT plt_chromatic_adapt(plt_srgb(1, 0, 0), 'd50', 'bradford');
```

```sql {"type":"duckfn","expect":"error"}
SELECT plt_adaptation_matrix('d65', 'd50', 'cat02');
```

## 3x3 矩阵

矩阵永远是恰好九个值的 `LIST(DOUBLE)`，**行优先**：

```text
[m11, m12, m13, m21, m22, m23, m31, m32, m33]
```

### `plt_matrix_identity()`

**签名**：`plt_matrix_identity() -> DOUBLE[]`

### `plt_matrix_scale(s1, s2, s3)`

**签名**：`plt_matrix_scale(s1 DOUBLE, s2 DOUBLE, s3 DOUBLE) -> DOUBLE[]`

对角线上是这三个值的对角矩阵 —— 它独立缩放每个分量。

```sql {"type":"duckfn","show":"table"}
SELECT plt_matrix_identity() AS identity,
       plt_matrix_scale(2, 0.5, 1) AS scale;
```

### `plt_matrix_multiply(a, b)`

**签名**：`plt_matrix_multiply(a DOUBLE[], b DOUBLE[]) -> DOUBLE[]`

这个乘积**先作用 `a`、再作用 `b`** —— 即 palette 的 `Matrix3::then` 顺序。要「先适应再缩放」，就把
适应矩阵放在前面：

```sql {"type":"duckfn","show":"table"}
SELECT plt_matrix_multiply(plt_matrix_identity(), plt_matrix_scale(2, 2, 2)) AS scaled,
       plt_matrix_multiply(plt_matrix_scale(2, 2, 2), plt_matrix_identity()) AS same;
```

### `plt_matrix_inverse(matrix)`

**签名**：`plt_matrix_inverse(matrix DOUBLE[]) -> DOUBLE[]`

### `plt_matrix_apply(matrix, x, y, z)`

**签名**：`plt_matrix_apply(matrix DOUBLE[], x DOUBLE, y DOUBLE, z DOUBLE) -> DOUBLE[]`

把矩阵作用到三个数上，返回三个结果 —— 不涉及颜色类型：

```sql {"type":"duckfn","show":"table"}
SELECT plt_matrix_apply(plt_matrix_scale(2, 3, 4), 1, 1, 1) AS scaled,
       plt_matrix_apply(plt_matrix_inverse(plt_matrix_scale(2, 4, 0.5)), 2, 4, 0.5) AS back_to_one;
```

### `plt_apply_matrix(matrix, color)`

**签名**：`plt_apply_matrix(matrix DOUBLE[], color plt_color) -> plt_color`

把矩阵作用到颜色在**当前 space** 里的三个分量上，保留 space 标签与 alpha。典型用法是 `xyz` 配一个
适应矩阵或三原色矩阵：

```sql {"type":"duckfn","show":"table"}
SELECT plt_components(plt_apply_matrix(plt_matrix_scale(0.5, 0.5, 0.5),
                                       plt_xyz(0.4, 0.3, 0.2))) AS halved,
       plt_space(plt_apply_matrix(plt_matrix_scale(0.5, 0.5, 0.5), plt_oklch(0.5, 0.1, 30))) AS still_oklch;
```

会报错的情况：列表不是九个数字，以及矩阵无法求逆：

```sql {"type":"duckfn","expect":"error"}
SELECT plt_matrix_apply([1, 0, 0, 0, 1, 0], 1, 2, 3);
```

## 编码：传递函数、三原色与矩阵

### `plt_encoding_spaces()`

**签名**：`plt_encoding_spaces() -> VARCHAR[]`

15 个 RGB space 名字加上三个 luma 名字 —— 也就是所有带传递函数的 space。

```sql {"type":"duckfn","show":"table"}
SELECT plt_encoding_spaces() AS spaces;
```

### `plt_transfer_function(space)`

**签名**：`plt_transfer_function(space VARCHAR) -> VARCHAR`

某个 space 用的是哪个传递函数。编码变体与线性变体会给出不同答案：

```sql {"type":"duckfn","show":"table"}
SELECT plt_transfer_function('srgb') AS srgb,
       plt_transfer_function('srgb_linear') AS srgb_linear,
       plt_transfer_function('dci_p3') AS dci_p3,
       plt_transfer_function('prophoto_rgb') AS prophoto,
       plt_transfer_function('rec2020') AS rec2020;
```

### `plt_transfer_functions()`

**签名**：`plt_transfer_functions() -> VARCHAR[]`

七个名字：`linear`、`gamma_2_2`、`srgb`、`adobe_rgb`、`prophoto_rgb`、`p3_gamma`、`rec_oetf`。

### `plt_decode(space, value)`

**签名**：`plt_decode(space VARCHAR, value DOUBLE) -> DOUBLE`

用一个 **已编码** 的分量值，按该 space 的传递函数求出线性光。这就是每个 RGB 分量进入 XYZ 时经历的
那一步：

```sql {"type":"duckfn","show":"table"}
SELECT plt_decode('srgb', 0.5) AS half_encoded,
       plt_decode('srgb', 1.0) AS one,
       plt_decode('srgb_linear', 0.5) AS already_linear;
```

### `plt_encode(space, value)`

**签名**：`plt_encode(space VARCHAR, value DOUBLE) -> DOUBLE`

反向操作：把一个线性值编回该 space 的编码。

```sql {"type":"duckfn","show":"table"}
SELECT plt_encode('srgb', 0.21404114048223255) AS back_to_half,
       plt_encode('srgb', plt_decode('srgb', 0.25)) AS round_trip;
```

### `plt_white_point_of(space)`

**签名**：`plt_white_point_of(space VARCHAR) -> VARCHAR`

某个 space 定义时参照的白点。它接受全部 44 个 space，不只是 RGB：

```sql {"type":"duckfn","show":"table"}
SELECT plt_white_point_of('srgb') AS srgb,
       plt_white_point_of('prophoto_rgb') AS prophoto,
       plt_white_point_of('xyz_d50') AS xyz_d50,
       plt_white_point_of('dci_p3') AS dci_p3,
       plt_white_point_of('oklch') AS oklch;
```

### `plt_rgb_primaries(space)`

**签名**：`plt_rgb_primaries(space VARCHAR) -> plt_color[]`

某个 RGB space 的红、绿、蓝三原色，每个都是一个 `yxy` 颜色（色度加亮度），顺序就是红绿蓝。luma
space 名字也被接受，它们与 sRGB 共享三原色：

```sql {"type":"duckfn","show":"table"}
SELECT plt_rgb_primaries('srgb') AS primaries,
       plt_components(plt_rgb_primaries('display_p3')[1]) AS p3_red;
```

### `plt_rgb_to_xyz_matrix(space)`

**签名**：`plt_rgb_to_xyz_matrix(space VARCHAR) -> DOUBLE[]`

该 space 的 3×3 RGB→XYZ 矩阵，行优先。这是转换中的线性部分：

```sql {"type":"duckfn","show":"table"}
SELECT plt_rgb_to_xyz_matrix('srgb') AS srgb,
       plt_rgb_to_xyz_matrix('rec2020') AS rec2020;
```

### `plt_xyz_to_rgb_matrix(space)`

**签名**：`plt_xyz_to_rgb_matrix(space VARCHAR) -> DOUBLE[]`

逆矩阵，XYZ→RGB，由上面那个算出来：

```sql {"type":"duckfn","show":"table"}
SELECT plt_xyz_to_rgb_matrix('srgb') AS xyz_to_srgb;
```

用这些零件复现一次转换 —— 解码分量、作用矩阵，再与 `plt_convert` 对比：

```sql {"type":"duckfn","show":"table"}
SELECT plt_matrix_apply(plt_rgb_to_xyz_matrix('srgb'),
                        plt_decode('srgb', 1), plt_decode('srgb', 0.5), plt_decode('srgb', 0)) AS by_hand,
       plt_components(plt_convert(plt_srgb(1, 0.5, 0), 'xyz')) AS converted;
```

没有三原色的 space 会报错：

```sql {"type":"duckfn","expect":"error"}
SELECT plt_rgb_primaries('lab');
```

```sql {"type":"duckfn","expect":"error"}
SELECT plt_decode('hsl', 0.5);
```
