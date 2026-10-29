---
title: 函数
sidebar_position: 3
description: duckfn_palette 注册的全部 SQL 函数，示例可直接在浏览器里运行。
---

# 函数

加载扩展后注册 145 个函数，全部以 `plt_` 开头。它们和 DuckDB 自带的函数一样工作：放在投影、
`WHERE` 或 `GROUP BY` 里都行，也能与内置函数自由组合。

## SQL 里的颜色

颜色是一个 `STRUCT`：空间标签、三个分量、alpha。

```text
STRUCT(space VARCHAR, c1 DOUBLE, c2 DOUBLE, c3 DOUBLE, alpha DOUBLE)
```

`plt_color_spaces()` 里的每个空间都有同名构造器，分量顺序与 palette 的 `Color::new` 一致：

```text
plt_srgb(red, green, blue)
plt_oklch(lightness, chroma, hue)
plt_lab(lightness, a, b)
plt_luma(luma)                          -- 只有一个分量
```

`plt_color(space, c1, c2, c3)` 用空间*名字*构造颜色，`plt_from_components(space, list)` 用列表给三个
分量，`plt_from_components_alpha` 用四个值 —— 三个分量加 alpha。`plt_alpha`、`plt_with_alpha`、
`plt_opaque` 读写 alpha，不动颜色本身。

```sql {"type":"duckfn","show":"table"}
SELECT plt_color_spaces() AS spaces;
```

```sql {"type":"duckfn","show":"table"}
SELECT plt_srgb(1, 0.5, 0) AS color,
       plt_space(plt_srgb(1, 0.5, 0)) AS space,
       plt_components(plt_srgb(1, 0.5, 0)) AS components,
       plt_alpha(plt_with_alpha(plt_srgb(1, 0.5, 0), 0.5)) AS alpha;
```

```sql {"type":"duckfn","show":"table"}
SELECT plt_to_hex(plt_from_components('srgb', [0.2, 0.4, 0.6])) AS built;
```

```sql {"type":"duckfn","show":"table"}
SELECT plt_to_hex(plt_from_components_alpha('srgb', [0.2, 0.4, 0.6, 0.5])) AS translucent;
```

## 转换

`plt_convert(color, space)` 把颜色移进另一个空间，并保留 alpha。结果带着目标空间的标签，可以继续
转换。`plt_convert_clamped` 会顺便把结果夹进目标空间的标准范围，`plt_clamp` 在当前空间里夹紧，
`plt_is_within_bounds` 回答夹紧是否会改变任何东西。

```sql {"type":"duckfn","show":"table"}
SELECT plt_to_hex(plt_srgb(1, 0.5, 0)) AS srgb,
       plt_to_hex(plt_convert(plt_srgb(1, 0.5, 0), 'oklch')) AS as_oklch,
       plt_to_hex(plt_convert(plt_srgb(1, 0.5, 0), 'xyz')) AS as_xyz;
```

```sql {"type":"duckfn","show":"table"}
SELECT plt_lab(50, 200, 200) AS out_of_range,
       plt_is_within_bounds(plt_lab(50, 200, 200)) AS within_bounds,
       plt_components(plt_clamp(plt_lab(50, 200, 200))) AS clamped;
```

## 变亮、变暗、饱和度与色相

这些操作只在部分空间里有定义，因为它们作用在特定类型的分量上。RGB 空间支持
`plt_lighten` / `plt_darken`；饱和度需要带色相的空间（`hsl`、`hsv`、`lch`、`okhsl` …），色相操作
同样需要带色相的空间。常见写法是先转换：

```sql {"type":"duckfn","show":"table"}
SELECT plt_to_hex(plt_lighten(plt_srgb(1, 0, 0), 0.2)) AS lighter,
       plt_to_hex(plt_darken(plt_srgb(1, 0, 0), 0.2)) AS darker;
```

```sql {"type":"duckfn","show":"table"}
SELECT plt_to_hex(plt_saturate(plt_convert(plt_srgb(1, 0, 0), 'hsl'), 0.2)) AS saturated,
       plt_to_hex(plt_shift_hue(plt_convert(plt_srgb(1, 0, 0), 'oklch'), 120)) AS shifted;
```

`plt_mix(a, b, factor)` 在两个颜色之间插值（在 `a` 的空间里）；`plt_premultiply` /
`plt_unpremultiply` 在直通 alpha 与预乘 alpha 之间转换。

```sql {"type":"duckfn","show":"table"}
SELECT plt_to_hex(plt_mix(plt_srgb(1, 0, 0), plt_srgb(0, 0, 1), 0.5)) AS purple;
```

## 混合与合成

`plt_blend(source, destination, mode)` 用 11 种可分离混合模式之一，
`plt_compose(source, destination, mode)` 用 6 种 Porter-Duff 算子之一。第一个参数是源、第二个是背景，
与 palette 一致。OpenGL 风格的自定义因子走 `plt_blend_equations`。

```sql {"type":"duckfn","show":"table"}
SELECT plt_blend_modes() AS blend_modes, plt_compose_modes() AS compose_modes;
```

```sql {"type":"duckfn","show":"table"}
SELECT plt_to_hex(plt_blend(plt_srgb(0.5, 0.2, 0.1), plt_srgb(0.4, 0.6, 0.8), 'multiply')) AS multiplied;
```

```sql {"type":"duckfn","show":"table"}
SELECT plt_to_hex(plt_compose(plt_with_alpha(plt_srgb(1, 0, 0), 0.5),
                              plt_with_alpha(plt_srgb(0, 0, 1), 0.5), 'over')) AS composited;
```

## 色差与对比度

`plt_difference(a, b, metric)` 计算色差；给第四个参数就在那个空间里算，而不是默认的 CIE L\*a\*b\*。
`plt_contrast` 是 WCAG 2.1 的对比度（1 到 21），`plt_contrast_check` 直接回答 WCAG 判定。

```sql {"type":"duckfn","show":"table"}
SELECT plt_difference_metrics() AS metrics, plt_contrast_criteria() AS criteria;
```

```sql {"type":"duckfn","show":"table"}
SELECT plt_difference(plt_lab(50, 2.6772, -79.7751), plt_lab(50, 0, -82.7485), 'ciede2000') AS delta_e;
```

```sql {"type":"duckfn","show":"table"}
SELECT plt_contrast(plt_srgb(1, 1, 1), plt_srgb(0, 0, 0)) AS contrast,
       plt_contrast_check(plt_srgb(1, 1, 1), plt_srgb(0, 0, 0), 'min_text') AS passes;
```

## 配色方案

配色函数需要带色相的颜色 —— 先转成 `oklch`（或其它色相空间）。`plt_complementary`、
`plt_split_complementary`、`plt_analogous`、`plt_triadic`、`plt_tetradic` 返回颜色列表；
`plt_harmony(color, scheme)` 按名字取一种。

```sql {"type":"duckfn","show":"table"}
SELECT plt_color_schemes() AS schemes;
```

```sql {"type":"duckfn","show":"table"}
SELECT plt_to_hex(plt_complementary(plt_convert(plt_srgb(1, 0, 0), 'oklch'))) AS complement;
```

```sql {"type":"duckfn","show":"table"}
SELECT to_json(list_transform(plt_harmony(plt_convert(plt_srgb(1, 0, 0), 'oklch'), 'triadic'),
                              lambda x: plt_to_hex(x))) AS triadic;
```

## 命名色、CSS 与十六进制

`plt_named(name)` 查 CSS 命名色，`plt_from_hex` 解析十六进制，`plt_to_hex` 序列化成短十六进制，
`plt_css_parse` / `plt_css_serialize` 正反处理完整的 CSS 颜色语法。`plt_css_is_valid` 回答字符串能否
解析，`plt_css_formats()` 列出可用的序列化格式。

```sql {"type":"duckfn","show":"table"}
SELECT plt_named_names() AS names;
```

```sql {"type":"duckfn","show":"table"}
SELECT plt_to_hex(plt_named('rebeccapurple')) AS named,
       plt_to_hex(plt_from_hex('#ff8000')) AS from_hex,
       plt_to_hex(plt_css_parse('rgb(255 128 0 / 50%)')) AS parsed;
```

```sql {"type":"duckfn","show":"table"}
SELECT plt_css_formats() AS formats,
       plt_css_serialize(plt_srgb(1, 0.5, 0), 'hex') AS as_hex,
       plt_css_serialize(plt_srgb(1, 0.5, 0), 'rgb') AS as_rgb;
```

## 白点、色适应、编码与矩阵

`plt_white_points()` 列出命名的白点；`plt_white_point_xyz` / `plt_white_point_xy` 返回它们的色度。
`plt_chromatic_adapt(color, target, method)` 在白点之间适应颜色（四参重载可显式给源白点），矩阵函数
则暴露它由哪些部件拼成。

```sql {"type":"duckfn","show":"table"}
SELECT plt_white_points() AS white_points;
```

```sql {"type":"duckfn","show":"table"}
SELECT plt_adaptation_methods() AS methods;
```

```sql {"type":"duckfn","show":"table"}
SELECT plt_to_hex(plt_chromatic_adapt(plt_xyz(0.4, 0.3, 0.2), 'd50', 'bradford')) AS adapted;
```

```sql {"type":"duckfn","show":"table"}
SELECT plt_adaptation_matrix('d65', 'd50', 'bradford')[1] AS first_entry;
```

`plt_encoding_spaces()` 列出带编码的 RGB 空间，每个编码都暴露自己的传递函数、白点与三原色。
`plt_decode` / `plt_encode` 按名字应用传递函数。

```sql {"type":"duckfn","show":"table"}
SELECT plt_transfer_function('srgb') AS transfer,
       plt_white_point_of('srgb') AS white_point,
       plt_decode('srgb', 0.5) AS decoded,
       plt_encode('srgb', 0.5) AS encoded;
```

```sql {"type":"duckfn","show":"table"}
SELECT plt_rgb_primaries('srgb') AS primaries,
       plt_rgb_to_xyz_matrix('srgb')[1] AS matrix_entry;
```

矩阵是行主序的九个 double。`plt_matrix_multiply(a, b)` 先作用 `a` 再作用 `b`；`plt_apply_matrix` 把矩阵
作用到颜色的分量上，但不改变空间。

```sql {"type":"duckfn","show":"table"}
SELECT plt_matrix_identity() AS identity;
```

```sql {"type":"duckfn","show":"table"}
SELECT plt_matrix_apply(plt_matrix_scale(2, 2, 2), 1, 2, 3) AS scaled;
```

## 8 位打包

`plt_pack(color)` 量化到 8 位 sRGB，按 ARGB 顺序打进 32 位整数；`plt_pack(color, order)` 与
`plt_unpack(value, order)` 把顺序作为参数。顺序有 `argb`、`rgba`、`abgr`、`bgra`。

```sql {"type":"duckfn","show":"table"}
SELECT plt_pack(plt_srgb(1, 0.5, 0)) AS packed,
       plt_to_hex(plt_unpack(plt_pack(plt_srgb(1, 0.5, 0)))) AS unpacked;
```

## 自定义观察条件的 CAM16

`plt_convert` 里的 `cam16_*` 空间始终使用 palette 的默认观察条件。`plt_cam16_*` 系列把它变成参数：
`plt_cam16_params` 从命名白点构造五个参数，`plt_cam16_params_xyz` 从数值白点构造，`from_xyz` /
`to_xyz` 成对在 `xyz` 与全属性 CAM16 或某个部分 CAM16 空间之间转换。

```sql {"type":"duckfn","show":"table"}
SELECT plt_cam16_surrounds() AS surrounds, plt_cam16_spaces() AS spaces;
```

```sql {"type":"duckfn","show":"table"}
SELECT plt_cam16_from_xyz(plt_xyz(0.4, 0.3, 0.2), plt_cam16_default_params()).lightness AS lightness,
       plt_to_hex(plt_cam16_to_xyz(plt_cam16_from_xyz(plt_xyz(0.4, 0.3, 0.2),
                                                     plt_cam16_default_params()),
                                   plt_cam16_default_params())) AS round_trip;
```

## colorous 预设

colorous 自带 d3 风格的渐变与分类色板。`plt_colorous_gradient` 在 `[0, 1]` 的某个位置取色，
`plt_colorous_palette` 均匀取 `n` 个颜色，`categorical` 一对返回整组色板。

```sql {"type":"duckfn","show":"table"}
SELECT plt_to_hex(plt_colorous_gradient('viridis', 0.5)) AS midpoint,
       to_json(list_transform(plt_colorous_palette('viridis', 3), lambda x: plt_to_hex(x))) AS sampled;
```

```sql {"type":"duckfn","show":"table"}
SELECT to_json(list_transform(plt_colorous_categorical('tableau10'), lambda x: plt_to_hex(x))) AS tableau10;
```

## 随机色

`plt_random_color(space, seed)` 按 palette 为该空间定义的均匀分布采样，`plt_random_colors(space, n,
seed)` 返回一个序列。同一个种子永远得到同样的结果；`plt_random_spaces()` 列出有分布的空间。

```sql {"type":"duckfn","show":"table"}
SELECT plt_to_hex(plt_random_colors('oklch', 3, 42)[1]) AS first,
       length(plt_random_colors('oklch', 3, 42)) AS count;
```

## 约定

- **NULL 进、NULL 出。** 任一参数是 NULL 就得到 NULL；palette 算不出的东西也是 NULL —— 不会是
  `NaN` 或 0。
- **非法参数让语句失败。** 错误信息以函数名开头，例如 `plt_convert: unknown color space 'nope'`。

```sql {"type":"duckfn","expect":"error"}
SELECT plt_convert(plt_srgb(1, 0, 0), 'nope');
```

- **名字会规范化**：大小写不敏感，连字符与空格视作下划线。

```sql {"type":"duckfn","show":"table"}
SELECT plt_to_hex(plt_convert(plt_srgb(1, 0.5, 0), 'OKLCH')) AS upper_case;
```
