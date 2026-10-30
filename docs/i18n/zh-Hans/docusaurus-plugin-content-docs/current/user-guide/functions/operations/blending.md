---
title: 混合与合成
sidebar_position: 2
description: 十一种可分离混合模式、六种 Porter-Duff 算子，以及 OpenGL 风格的自定义因子，顺序与 palette 的 source/destination 一致。
---

# 混合与合成

三个函数都是**源在前、目标在后** —— 与 palette 的 `source.multiply(destination)` 是同一顺序。第二个
颜色若在不同的 space，会先转换到第一个颜色的 space，结果也带着那个 space。

由于这些模式是逐分量定义的，它们需要一个类似 RGB 的刺激空间。各函数接受的 space 并不相同：

| 函数 | 第一个参数接受的 space |
| --- | --- |
| `plt_blend` | 15 个 RGB space、`xyz`、`xyz_d50`、`luma`、`luma_linear`、`luma_gamma`、`lms`、`lms_von_kries` |
| `plt_compose` | 同上，外加 `lab`、`luv`、`yxy`、`oklab`、`cam16_ucs_jab` |
| `plt_blend_equations` | 与 `plt_compose` 相同 |

带色相的 space（`hsl`、`lch`、`oklch`、CAM16 JCH/QCH/…）会被拒绝 —— 逐分量混合一个色相角没有意义。
先转换：

```sql {"type":"duckfn","expect":"error"}
SELECT plt_blend(plt_hsl(0, 1, 0.5), plt_hsl(120, 1, 0.5), 'multiply');
```

```sql {"type":"duckfn","show":"table"}
SELECT plt_to_hex(plt_blend(plt_convert(plt_hsl(0, 1, 0.5), 'srgb'),
                            plt_convert(plt_hsl(120, 1, 0.5), 'srgb'),
                            'multiply')) AS multiplied;
```

## `plt_blend(source, destination, mode)`

**签名**：`plt_blend(source plt_color, destination plt_color, mode VARCHAR) -> plt_color`

11 种可分离模式，与 CSS/PDF 的合成模型一致：

| 模式 | 一句话结果 |
| --- | --- |
| `multiply` | 更暗：`source × destination` |
| `screen` | 更亮：两个补值相乘，再取补 |
| `overlay` | 视目标而定，用 multiply 或 screen |
| `darken` | 逐分量取两者中更暗的 |
| `lighten` | 逐分量取两者中更亮的 |
| `dodge` | 把目标朝白色提亮 |
| `burn` | 把目标朝黑色压暗 |
| `hard_light` | 交换 source 与 destination 的 overlay |
| `soft_light` | 更柔和的 hard light |
| `difference` | 绝对差值 |
| `exclusion` | 对比度更低的 difference |

```sql {"type":"duckfn","show":"table"}
SELECT plt_blend_modes() AS modes;
```

```sql {"type":"duckfn","show":"table"}
SELECT plt_to_hex(plt_blend(plt_srgb(0.5, 0.2, 0.1), plt_srgb(0.4, 0.6, 0.8), 'multiply')) AS multiply,
       plt_to_hex(plt_blend(plt_srgb(0.5, 0.2, 0.1), plt_srgb(0.4, 0.6, 0.8), 'screen')) AS screen,
       plt_to_hex(plt_blend(plt_srgb(0.5, 0.2, 0.1), plt_srgb(0.4, 0.6, 0.8), 'overlay')) AS overlay;
```

结果的 alpha 由混合本身决定：源与目标都不透明时结果也不透明，源半透明时目标会透出来：

```sql {"type":"duckfn","show":"table"}
SELECT plt_blend(plt_with_alpha(plt_srgb(0.5, 0.2, 0.1), 0.5),
                 plt_with_alpha(plt_srgb(0.4, 0.6, 0.8), 0.5),
                 'hard-light') AS result;
```

模式名会规范化：大小写不敏感，`-` 或空格视作 `_`，所以 `'Hard Light'` 等同于 `'hard_light'`。未知的
名字会让语句失败：

```sql {"type":"duckfn","expect":"error"}
SELECT plt_blend(plt_srgb(1, 0, 0), plt_srgb(0, 0, 1), 'glow');
```

## `plt_compose(source, destination, operator)`

**签名**：`plt_compose(source plt_color, destination plt_color, operator VARCHAR) -> plt_color`

六种 Porter-Duff 算子：

| 算子 | 保留 |
| --- | --- |
| `over` | 源覆盖目标（通常的 alpha 合成） |
| `inside` | 源落在目标内部的部分 |
| `outside` | 源落在目标外部的部分 |
| `atop` | 源覆盖目标，但限于目标的区域 |
| `xor` | 两者互不重叠的部分 |
| `plus` | 两者之和，alpha 也算在内 |

```sql {"type":"duckfn","show":"table"}
SELECT plt_compose_modes() AS operators;
```

```sql {"type":"duckfn","show":"table"}
SELECT plt_to_hex(plt_compose(plt_with_alpha(plt_srgb(1, 0, 0), 0.5),
                              plt_with_alpha(plt_srgb(0, 0, 1), 0.5), 'over')) AS over,
       plt_to_hex(plt_compose(plt_with_alpha(plt_srgb(1, 0, 0), 0.5),
                              plt_with_alpha(plt_srgb(0, 0, 1), 0.5), 'xor')) AS xor_,
       plt_compose(plt_with_alpha(plt_srgb(1, 0, 0), 0.5),
                   plt_with_alpha(plt_srgb(0, 0, 1), 0.5), 'plus') AS plus_alpha;
```

`plt_compose` 还能用在 `lab`、`luv`、`yxy`、`oklab` 与 `cam16_ucs_jab` 里，`plt_blend` 则不行：

```sql {"type":"duckfn","show":"table"}
SELECT plt_compose(plt_with_alpha(plt_oklab(0.5, 0.1, 0.1), 0.5),
                   plt_with_alpha(plt_oklab(0.7, -0.1, 0.1), 0.5), 'over') AS in_oklab;
```

## `plt_blend_equations(source, destination, source_factor, destination_factor)`

**签名**：`plt_blend_equations(source plt_color, destination plt_color, source_factor VARCHAR, destination_factor VARCHAR) -> plt_color`

OpenGL 风格的形式：由你决定源与目标各有多少进入结果。十个因子中任意两两组合都接受：

```sql {"type":"duckfn","show":"table"}
SELECT plt_blend_parameters() AS factors;
```

```sql {"type":"duckfn","show":"table"}
SELECT plt_to_hex(plt_blend_equations(plt_with_alpha(plt_srgb(1, 0, 0), 0.5),
                                      plt_with_alpha(plt_srgb(0, 0, 1), 0.5),
                                      'source_alpha', 'one_minus_source_alpha')) AS blended;
```

`'source_alpha'` / `'one_minus_source_alpha'` 复现了普通的 `over` 合成，用它来检验一对因子的行为是否
符合预期是个好办法：

```sql {"type":"duckfn","show":"table"}
SELECT plt_to_hex(plt_compose(src, dst, 'over')) AS compose_over,
       plt_to_hex(plt_blend_equations(src, dst, 'source_alpha', 'one_minus_source_alpha')) AS equations
FROM (SELECT plt_with_alpha(plt_srgb(1, 0, 0), 0.5) AS src,
             plt_with_alpha(plt_srgb(0, 0, 1), 0.5) AS dst);
```

未知的因子或算子会报错：

```sql {"type":"duckfn","expect":"error"}
SELECT plt_blend_equations(plt_srgb(1, 0, 0), plt_srgb(0, 0, 1), 'half', 'one');
```

```sql {"type":"duckfn","expect":"error"}
SELECT plt_compose(plt_srgb(1, 0, 0), plt_srgb(0, 0, 1), 'around');
```

## 列出支持的名字

### `plt_blend_modes()`

**签名**：`plt_blend_modes() -> VARCHAR[]`

`plt_blend` 接受的 11 个模式名，顺序固定。可以用它驱动对一张颜色表的 `list_transform`，而不必把列表
写死在查询里。

### `plt_compose_modes()`

**签名**：`plt_compose_modes() -> VARCHAR[]`

`plt_compose` 接受的六个算子名。

### `plt_blend_parameters()`

**签名**：`plt_blend_parameters() -> VARCHAR[]`

`plt_blend_equations` 接受的十个因子名：`zero`、`one`、`source_color`、
`one_minus_source_color`、`destination_color`、`one_minus_destination_color`、`source_alpha`、
`one_minus_source_alpha`、`destination_alpha`、`one_minus_destination_alpha`。

```sql {"type":"duckfn","show":"table"}
SELECT unnest(plt_blend_modes()) AS mode, 'blend' AS used_by
FROM (SELECT 1)
UNION ALL
SELECT unnest(plt_compose_modes()), 'compose'
UNION ALL
SELECT unnest(plt_blend_parameters()), 'equations';
```
