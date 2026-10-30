---
title: 颜色调整
sidebar_position: 1
description: 混合颜色、变亮与变暗、增饱和与去饱和、旋转或设置色相、预乘 alpha —— 都在颜色自身的 space 里完成。
---

# 颜色调整

这些函数改的是单个颜色，从不做转换：结果保留输入的 space 与 alpha，所以
`plt_lighten(plt_hsl(...), 0.2)` 仍然是一个 `hsl` 值。每个操作都只对拥有它所作用分量的 space 有
定义 —— RGB 空间没有可旋转的色相，HSL 也没有 a/b 轴 —— 所以各函数支持的 space 列表并不相同。

space 不支持该操作时，查询会报错；错误信息会说明是哪个函数失败，并建议先做转换：

```sql {"type":"duckfn","expect":"error"}
SELECT plt_saturate(plt_srgb(1, 0, 0), 0.2);
```

修好它只差一次 `plt_convert`：

```sql {"type":"duckfn","show":"table"}
SELECT plt_to_hex(plt_saturate(plt_convert(plt_srgb(1, 0, 0), 'hsl'), 0.2)) AS saturated;
```

## 混合

### `plt_mix(a, b, factor)`

**签名**：`plt_mix(a plt_color, b plt_color, factor DOUBLE) -> plt_color`

在 **`a` 的 space** 里做线性插值：`factor` 为 0 得到 `a`，为 1 得到 `b`，0.5 是两者的中点。
`factor` 会被夹进 0..1。`b` 先转换到 `a` 的 space，alpha 与各分量一起插值。混合在全部 44 个 space
里都有定义，包括带色相的 space：

```sql {"type":"duckfn","show":"table"}
SELECT plt_to_hex(plt_mix(plt_srgb(1, 0, 0), plt_srgb(0, 0, 1), 0.5)) AS srgb_mid,
       plt_to_hex(plt_mix(plt_convert(plt_srgb(1, 0, 0), 'oklab'),
                          plt_convert(plt_srgb(0, 0, 1), 'oklab'),
                          0.5)) AS oklab_mid;
```

```sql {"type":"duckfn","show":"table"}
SELECT plt_mix(plt_with_alpha(plt_srgb(1, 0, 0), 0),
               plt_with_alpha(plt_srgb(1, 0, 0), 1),
               0.25) AS quarter_alpha;
```

## 变亮与变暗

两种形式都保留原 space。按比例的形式（`plt_lighten`、`plt_darken`）把亮度朝该 space 的最大值或最小值
缩放；固定量的形式（`plt_lighten_fixed`、`plt_darken_fixed`）按一个绝对量平移它。两者都接受 0..1 的
值，取 0 时颜色不变。

四个函数共同支持的 space：全部 15 个 RGB 变体、`hsl`、`hsv`、`hwb`、`hsluv`、`lab`、`lch`、
`luv`、`lchuv`、`xyz`、`xyz_d50`、`yxy`、`luma`、`luma_linear`、`luma_gamma`、`oklab`、`oklch`、
`okhsl`、`okhsv`、`okhwb`、`cam16_ucs_jab` 与 `cam16_ucs_jmh`。

### `plt_lighten(color, factor)`

**签名**：`plt_lighten(color plt_color, factor DOUBLE) -> plt_color`

```sql {"type":"duckfn","show":"table"}
SELECT plt_to_hex(plt_lighten(plt_srgb(1, 0, 0), 0.2)) AS lighter,
       plt_to_hex(plt_lighten(plt_hsl(0, 1, 0.4), 0.2)) AS lighter_hsl,
       plt_components(plt_lighten(plt_oklch(0.5, 0.1, 30), 0.2)) AS lighter_oklch;
```

### `plt_lighten_fixed(color, amount)`

**签名**：`plt_lighten_fixed(color plt_color, amount DOUBLE) -> plt_color`

```sql {"type":"duckfn","show":"table"}
SELECT plt_components(plt_lighten_fixed(plt_lab(50, 20, -30), 0.1)) AS raised_lab,
       plt_to_hex(plt_lighten_fixed(plt_hsl(0, 1, 0.4), 0.2)) AS raised_hsl;
```

### `plt_darken(color, factor)`

**签名**：`plt_darken(color plt_color, factor DOUBLE) -> plt_color`

```sql {"type":"duckfn","show":"table"}
SELECT plt_to_hex(plt_darken(plt_srgb(1, 0, 0), 0.2)) AS darker,
       plt_to_hex(plt_darken(plt_hsv(0, 1, 0.6), 0.3)) AS darker_hsv;
```

### `plt_darken_fixed(color, amount)`

**签名**：`plt_darken_fixed(color plt_color, amount DOUBLE) -> plt_color`

```sql {"type":"duckfn","show":"table"}
SELECT plt_components(plt_darken_fixed(plt_lab(50, 20, -30), 0.1)) AS lowered_lab;
```

## 饱和度

按比例的形式把饱和度或 chroma 朝该 space 的最大值（`plt_saturate`）或最小值（`plt_desaturate`）缩放；
固定量的形式则加上或减去一个绝对量。两种形式的参数都在 0..1。

支持的 space：`hsl`、`hsv`、`hsluv`、`lch`、`lchuv`、`okhsl`、`okhsv` 与 `cam16_ucs_jmh`。`oklch`
本身不在其中 —— 想缩放 CIE chroma 就转成 `lch`，想缩放感知饱和度就留在 `okhsl` / `okhsv` 里。

### `plt_saturate(color, factor)`

**签名**：`plt_saturate(color plt_color, factor DOUBLE) -> plt_color`

```sql {"type":"duckfn","show":"table"}
SELECT plt_to_hex(plt_saturate(plt_hsl(0, 0.5, 0.5), 0.5)) AS more_saturated,
       plt_components(plt_saturate(plt_lch(50, 20, 40), 0.5)) AS more_chroma;
```

### `plt_saturate_fixed(color, amount)`

**签名**：`plt_saturate_fixed(color plt_color, amount DOUBLE) -> plt_color`

```sql {"type":"duckfn","show":"table"}
SELECT plt_components(plt_saturate_fixed(plt_hsl(0, 0.4, 0.5), 0.2)) AS fixed_hsl,
       plt_components(plt_saturate_fixed(plt_okhsl(0, 0.4, 0.5), 0.2)) AS fixed_okhsl;
```

### `plt_desaturate(color, factor)`

**签名**：`plt_desaturate(color plt_color, factor DOUBLE) -> plt_color`

```sql {"type":"duckfn","show":"table"}
SELECT plt_to_hex(plt_desaturate(plt_hsl(0, 0.8, 0.5), 0.5)) AS muted,
       plt_to_hex(plt_desaturate(plt_convert(plt_srgb(1, 0, 0), 'hsl'), 1)) AS gray;
```

### `plt_desaturate_fixed(color, amount)`

**签名**：`plt_desaturate_fixed(color plt_color, amount DOUBLE) -> plt_color`

```sql {"type":"duckfn","show":"table"}
SELECT plt_components(plt_desaturate_fixed(plt_hsl(0, 0.8, 0.5), 0.3)) AS less_saturated;
```

## 色相

### `plt_shift_hue(color, degrees)`

**签名**：`plt_shift_hue(color plt_color, degrees DOUBLE) -> plt_color`

把色相旋转任意实数度；结果会回绕到 0..360。

```sql {"type":"duckfn","show":"table"}
SELECT plt_to_hex(plt_shift_hue(plt_hsl(120, 1, 0.5), 120)) AS shifted,
       plt_to_hex(plt_shift_hue(plt_convert(plt_srgb(1, 0, 0), 'oklch'), -60)) AS backwards;
```

### `plt_with_hue(color, degrees)`

**签名**：`plt_with_hue(color plt_color, degrees DOUBLE) -> plt_color`

设置色相，而不是在原有色相上累加。

```sql {"type":"duckfn","show":"table"}
SELECT plt_to_hex(plt_with_hue(plt_hsl(120, 1, 0.5), 240)) AS blue,
       plt_components(plt_with_hue(plt_oklch(0.6, 0.1, 30), 200)) AS set_oklch;
```

### `plt_hue(color)`

**签名**：`plt_hue(color plt_color) -> DOUBLE`

以度返回色相，0..360。灰色返回 0。它的适用范围比色相修改函数更广：所有带色相的 space、所有 RGB
space，外加 `lab`、`luv` 与 `oklab` —— 后三者里色相是 a/b 或 u/v 这一对分量的夹角。

```sql {"type":"duckfn","show":"table"}
SELECT plt_hue(plt_srgb(1, 0, 0)) AS red,
       plt_hue(plt_oklab(0.5, 0.1, 0.1)) AS oklab_angle,
       plt_hue(plt_srgb(0.5, 0.5, 0.5)) AS gray;
```

并非每个 space 都有色相；`luma`、`xyz`、`lms` 之类会被拒绝：

```sql {"type":"duckfn","expect":"error"}
SELECT plt_hue(plt_luma(0.5));
```

## 预乘 alpha

### `plt_premultiply(color)`

**签名**：`plt_premultiply(color plt_color) -> plt_color`

把各分量乘以 alpha；alpha 本身不变。alpha 必须在 0..1。

```sql {"type":"duckfn","show":"table"}
SELECT plt_premultiply(plt_with_alpha(plt_srgb(1, 0.5, 0.25), 0.5)) AS premultiplied;
```

### `plt_unpremultiply(color)`

**签名**：`plt_unpremultiply(color plt_color) -> plt_color`

把各分量除以 alpha。alpha 为 0 时分量返回 0 —— 这是 palette 的定义，而不是错误。

```sql {"type":"duckfn","show":"table"}
SELECT plt_unpremultiply(plt_premultiply(plt_with_alpha(plt_srgb(1, 0.5, 0.25), 0.5))) AS round_trip,
       plt_unpremultiply(plt_with_alpha(plt_srgb(1, 0.5, 0.25), 0)) AS zero_alpha;
```

两者共同支持的 space：全部 15 个 RGB 变体、`lab`、`luv`、`xyz`、`xyz_d50`、`yxy`、`luma`、
`luma_linear`、`luma_gamma`、`oklab`、`cam16_ucs_jab`、`lms` 与 `lms_von_kries`。

```sql {"type":"duckfn","expect":"error"}
SELECT plt_premultiply(plt_with_alpha(plt_srgb(1, 0, 0), 1.5));
```

```sql {"type":"duckfn","expect":"error"}
SELECT plt_premultiply(plt_hsl(120, 0.5, 0.5));
```
