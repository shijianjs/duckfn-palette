---
title: 颜色与空间
sidebar_position: 1
description: plt_color 值、它的读取函数、alpha 的处理，以及 44 个颜色空间构造器各自的单位与范围。
---

# 颜色与空间

## 颜色值

每个函数都接收并返回同一个 `STRUCT`：

```text
STRUCT(space VARCHAR, c1 DOUBLE, c2 DOUBLE, c3 DOUBLE, alpha DOUBLE)
```

没有单独的 `hsl` 类型或 `lab` 类型 —— 值里的空间名决定了 `c1..c3` 该怎么读。这正是 `plt_convert` 得以
成立的原因，也是同一列里能放不同空间颜色的原因。

```sql {"type":"duckfn","show":"table"}
SELECT plt_srgb(1, 0.5, 0) AS color,
       plt_space(plt_srgb(1, 0.5, 0)) AS space,
       plt_components(plt_srgb(1, 0.5, 0)) AS components,
       plt_alpha(plt_srgb(1, 0.5, 0)) AS alpha;
```

## 通用构造

### `plt_color(space, c1, c2, c3)`

**签名**：`plt_color(space VARCHAR, c1 DOUBLE, c2 DOUBLE, c3 DOUBLE) -> plt_color`

按空间名在任意受支持的空间里构造颜色。alpha 默认为 1。空间名接受别名，例如 `xyz-d65`、`p3`、
`prophoto`、`cie-lab`、`ok-lch` 或 `grey`，大小写不限。

```sql {"type":"duckfn","show":"table"}
SELECT plt_convert(plt_color('lab', 50, 20, -30), 'srgb') AS from_name,
       plt_color('P3', 1, 0, 0).space AS alias;
```

### `plt_color(space, c1, c2, c3, alpha)`

**签名**：`plt_color(space VARCHAR, c1 DOUBLE, c2 DOUBLE, c3 DOUBLE, alpha DOUBLE) -> plt_color`

同上，但显式给出 alpha。

```sql {"type":"duckfn","show":"table"}
SELECT plt_color('oklch', 0.7, 0.1, 120, 0.5) AS translucent;
```

### `plt_from_components(space, components)`

**签名**：`plt_from_components(space VARCHAR, components DOUBLE[]) -> plt_color`

用一个恰好含三个分量的列表构造颜色；alpha 默认为 1。它是 `plt_components` 的逆操作，所以颜色经过
一个列表往返之后仍然完好。

```sql {"type":"duckfn","show":"table"}
SELECT plt_to_hex(plt_from_components('srgb', [0.2, 0.4, 0.6])) AS rebuilt;
```

### `plt_from_components_alpha(space, components)`

**签名**：`plt_from_components_alpha(space VARCHAR, components DOUBLE[]) -> plt_color`

接收一个恰好含四个值的列表：三个分量加 alpha。

```sql {"type":"duckfn","show":"table"}
SELECT plt_to_hex(plt_from_components_alpha('srgb', [0.2, 0.4, 0.6, 0.5])) AS translucent;
```

## 读取颜色

### `plt_space(color)`

**签名**：`plt_space(color plt_color) -> VARCHAR`

规范的空间名，也就是 `plt_color_spaces()` 列出的那些。它从不返回别名。

```sql {"type":"duckfn","show":"table"}
SELECT plt_space(plt_color('OkLcH', 0.5, 0.1, 120)) AS canonical;
```

### `plt_components(color)`

**签名**：`plt_components(color plt_color) -> DOUBLE[]`

按空间的自然顺序给出全部三个分量。对一维的 `luma*` 空间，列表始终是 `[value, 0, 0]`。

```sql {"type":"duckfn","show":"table"}
SELECT plt_components(plt_hsl(120, 0.5, 0.4)) AS hsl,
       plt_components(plt_lch(50, 40, 120)) AS lch,
       plt_components(plt_luma(0.5)) AS luma;
```

### `plt_component(color, index)`

**签名**：`plt_component(color plt_color, index BIGINT) -> DOUBLE`

取一个分量，下标从 1 开始。`index` 必须是 1、2 或 3。

```sql {"type":"duckfn","show":"table"}
SELECT plt_component(plt_srgb(0.1, 0.2, 0.3), 1) AS red,
       plt_component(plt_srgb(0.1, 0.2, 0.3), 3) AS blue;
```

### `plt_alpha(color)`

**签名**：`plt_alpha(color plt_color) -> DOUBLE`

alpha 通道，从 0（透明）到 1（不透明）。

```sql {"type":"duckfn","show":"table"}
SELECT plt_alpha(plt_srgb(1, 0, 0)) AS opaque,
       plt_alpha(plt_color('srgb', 1, 0, 0, 0.25)) AS translucent;
```

### `plt_with_alpha(color, alpha)`

**签名**：`plt_with_alpha(color plt_color, alpha DOUBLE) -> plt_color`

替换 alpha，空间与分量保持不变。

```sql {"type":"duckfn","show":"table"}
SELECT plt_with_alpha(plt_srgb(1, 0, 0), 0.5) AS half_transparent;
```

### `plt_opaque(color)`

**签名**：`plt_opaque(color plt_color) -> plt_color`

把 alpha 设为 1。

```sql {"type":"duckfn","show":"table"}
SELECT plt_alpha(plt_opaque(plt_color('srgb', 1, 0, 0, 0.25))) AS alpha;
```

### `plt_color_spaces()`

**签名**：`plt_color_spaces() -> VARCHAR[]`

全部 44 个规范空间名，按文档顺序排列。

```sql {"type":"duckfn","show":"table"}
SELECT list_count(plt_color_spaces()) AS count, plt_color_spaces() AS spaces;
```

## 每个空间一个构造器

`plt_color_spaces()` 里的每个名字都有一个同名构造器。它们都接收 `DOUBLE` 分量，顺序与 palette 的
`Color::new` 一致，并把 alpha 设为 1。分量按传入的值原样存储（不做裁剪）；要把它们带进范围，用
`plt_clamp` 或 `plt_convert_clamped`。

### RGB 空间（编码与线性）

| 构造器 | 分量 | 说明 |
| --- | --- | --- |
| `plt_srgb(r, g, b)` | 红、绿、蓝 0..1 | 伽马编码的 sRGB，默认输出空间 |
| `plt_srgb_linear(r, g, b)` | 红、绿、蓝 0..1 | 线性光 sRGB |
| `plt_srgb_gamma(r, g, b)` | 红、绿、蓝 0..1 | 伽马 2.2 编码（palette 已废弃的 `GammaSrgb`，仍然暴露） |
| `plt_adobe_rgb(r, g, b)` | 红、绿、蓝 0..1 | Adobe RGB (1998) |
| `plt_adobe_rgb_linear(r, g, b)` | 红、绿、蓝 0..1 | 线性光 Adobe RGB |
| `plt_display_p3(r, g, b)` | 红、绿、蓝 0..1 | Display P3，D65，sRGB 传递函数 |
| `plt_display_p3_linear(r, g, b)` | 红、绿、蓝 0..1 | 线性光 Display P3 |
| `plt_dci_p3(r, g, b)` | 红、绿、蓝 0..1 | DCI-P3，影院白点，伽马 2.6 |
| `plt_dci_p3_linear(r, g, b)` | 红、绿、蓝 0..1 | 线性光 DCI-P3 |
| `plt_prophoto_rgb(r, g, b)` | 红、绿、蓝 0..1 | ProPhoto RGB，D50，伽马 1.8 |
| `plt_prophoto_rgb_linear(r, g, b)` | 红、绿、蓝 0..1 | 线性光 ProPhoto RGB |
| `plt_rec709(r, g, b)` | 红、绿、蓝 0..1 | Rec. 709 三原色与传递函数 |
| `plt_rec709_linear(r, g, b)` | 红、绿、蓝 0..1 | 线性光 Rec. 709 |
| `plt_rec2020(r, g, b)` | 红、绿、蓝 0..1 | Rec. 2020 三原色与传递函数 |
| `plt_rec2020_linear(r, g, b)` | 红、绿、蓝 0..1 | 线性光 Rec. 2020 |

```sql {"type":"duckfn","show":"table"}
SELECT plt_to_hex(plt_srgb(1, 0.5, 0)) AS srgb,
       plt_to_hex(plt_display_p3(1, 0.5, 0)) AS display_p3,
       plt_rgb_to_xyz_matrix('rec2020') AS rec2020_matrix;
```

### 色相在前的空间

这些空间第一个分量是色相。色相以度为单位，可以是负数，也可以大于 360。

| 构造器 | 分量 | 范围 |
| --- | --- | --- |
| `plt_hsl(h, s, l)` | 色相、饱和度、亮度 | 色相为度；s 与 l 0..1 |
| `plt_hsv(h, s, v)` | 色相、饱和度、明度 | 色相为度；s 与 v 0..1 |
| `plt_hwb(h, w, b)` | 色相、白度、黑度 | 色相为度；w 与 b 0..1 |
| `plt_hsluv(h, s, l)` | 色相、饱和度、亮度 | 色相为度；s 与 l 0..100 |
| `plt_okhsl(h, s, l)` | 色相、饱和度、亮度 | 色相为度；s 与 l 0..1 |
| `plt_okhsv(h, s, v)` | 色相、饱和度、明度 | 色相为度；s 与 v 0..1 |
| `plt_okhwb(h, w, b)` | 色相、白度、黑度 | 色相为度；w 与 b 0..1 |

```sql {"type":"duckfn","show":"table"}
SELECT plt_to_hex(plt_hsl(120, 0.5, 0.5)) AS hsl,
       plt_to_hex(plt_okhsl(120, 0.5, 0.5)) AS okhsl,
       plt_to_hex(plt_hsluv(120, 50, 50)) AS hsluv;
```

### CIE 空间

| 构造器 | 分量 | 范围 |
| --- | --- | --- |
| `plt_lab(l, a, b)` | L\*、a\*、b\* | L\* 0..100；a\* 与 b\* 大致 -128..127 |
| `plt_lch(l, c, h)` | L\*、彩度、色相 | L\* 0..100；c ≥ 0；色相为度 |
| `plt_luv(l, u, v)` | L\*、u\*、v\* | L\* 0..100 |
| `plt_lchuv(l, c, h)` | L\*、彩度、色相 | L\* 0..100；c ≥ 0；色相为度 |
| `plt_xyz(x, y, z)` | X、Y、Z | D65 白点；白色时 Y = 1 |
| `plt_xyz_d50(x, y, z)` | X、Y、Z | D50 白点（转换时用 Bradford 做色适应） |
| `plt_yxy(x, y, luminance)` | 色度 x、y，亮度 | x 与 y 是色度，亮度 ≥ 0 |

```sql {"type":"duckfn","show":"table"}
SELECT plt_to_hex(plt_lab(50, 20, -30)) AS lab,
       plt_to_hex(plt_lch(50, 40, 120)) AS lch,
       plt_to_hex(plt_xyz(0.5, 0.4, 0.3)) AS xyz;
```

### 感知 OK 空间

| 构造器 | 分量 | 范围 |
| --- | --- | --- |
| `plt_oklab(l, a, b)` | 亮度、a、b | l 0..1；a 与 b 名义上 -1..1 |
| `plt_oklch(l, c, h)` | 亮度、彩度、色相 | l 0..1；c ≥ 0；色相为度 |

```sql {"type":"duckfn","show":"table"}
SELECT plt_to_hex(plt_oklab(0.5, 0.1, 0.1)) AS oklab,
       plt_to_hex(plt_oklch(0.5, 0.1, 120)) AS oklch;
```

### CAM16 空间（默认观察条件）

这些空间使用 palette 的默认条件：D65、适应亮度 40、背景亮度 0.2、平均环绕，并自动 discounting。
要用自己的条件，请用 [CAM16 函数](../color-science/cam16.md)。

| 构造器 | 分量 |
| --- | --- |
| `plt_cam16_jch(j, c, h)` | 亮度 J、彩度 C、色相 |
| `plt_cam16_jmh(j, m, h)` | 亮度 J、色彩度 M、色相 |
| `plt_cam16_jsh(j, s, h)` | 亮度 J、饱和度 s、色相 |
| `plt_cam16_qch(q, c, h)` | 明度 Q、彩度 C、色相 |
| `plt_cam16_qmh(q, m, h)` | 明度 Q、色彩度 M、色相 |
| `plt_cam16_qsh(q, s, h)` | 明度 Q、饱和度 s、色相 |
| `plt_cam16_ucs_jab(j, a, b)` | CAM16-UCS J′、a′、b′ |
| `plt_cam16_ucs_jmh(j, m, h)` | CAM16-UCS J′、M′、色相 |

```sql {"type":"duckfn","show":"table"}
SELECT plt_to_hex(plt_cam16_jch(50, 40, 120)) AS jch,
       plt_to_hex(plt_cam16_ucs_jmh(50, 40, 120)) AS ucs_jmh;
```

### LMS 空间

| 构造器 | 分量 |
| --- | --- |
| `plt_lms(l, m, s)` | 长、中、短（Bradford 矩阵，D65） |
| `plt_lms_von_kries(l, m, s)` | 长、中、短（von Kries / Hunt-Pointer-Estevez 矩阵，D65） |

```sql {"type":"duckfn","show":"table"}
SELECT plt_to_hex(plt_lms(0.5, 0.4, 0.3)) AS lms,
       plt_to_hex(plt_lms_von_kries(0.5, 0.4, 0.3)) AS von_kries;
```

### 一维亮度空间

`plt_luma`、`plt_luma_linear` 与 `plt_luma_gamma` 接收单个 0..1 的值。剩下两个分量都是 0。这些空间是
有损的：把一个颜色转进去再转回来，得到的是同样亮度的灰色，而不是原来的颜色。

```sql {"type":"duckfn","show":"table"}
SELECT plt_luma(0.5) AS luma,
       plt_components(plt_convert(plt_srgb(0.2, 0.4, 0.6), 'luma')) AS from_color,
       plt_to_hex(plt_convert(plt_srgb(0.2, 0.4, 0.6), 'luma')) AS gray;
```

## 错误

```sql {"type":"duckfn","expect":"error"}
SELECT plt_color('not-a-space', 1, 2, 3);
```

```sql {"type":"duckfn","expect":"error"}
SELECT plt_component(plt_srgb(1, 0, 0), 4);
```

```sql {"type":"duckfn","expect":"error"}
SELECT plt_from_components('srgb', [1, 2]);
```
