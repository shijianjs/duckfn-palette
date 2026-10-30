---
title: 概览
sidebar_position: 1
description: 160 个 plt_ 函数如何划分，每个函数都遵守的三条规则，以及各类函数该去哪里查。
---

# 函数参考概览

加载 `duckfn_palette` 会注册 **160 个名字 / 169 个签名**，全部位于 `plt_` 前缀下。它们就是普通的
DuckDB 标量函数：放在投影、`WHERE` 子句、`GROUP BY`、`LATERAL` 连接或列表 lambda 里，都和一个
内置函数没有区别。

## 它们操作的值

颜色是单个 `STRUCT`，所以它能待在列里，能被比较、分组，也能传给任何其它函数：

```text
STRUCT(space VARCHAR, c1 DOUBLE, c2 DOUBLE, c3 DOUBLE, alpha DOUBLE)
```

`space` 是 `plt_color_spaces()` 里 44 个名字中的一个；`c1..c3` 是该空间的分量，顺序与 palette 的
`Color::new` 一致；`alpha` 从 0（透明）到 1（不透明），转换时原样带过去。

```sql {"type":"duckfn","show":"table"}
SELECT plt_color_spaces() AS spaces;
```

## 处处适用的三条规则

- **NULL 进，NULL 出。** 任一参数是 NULL 就让整个调用变成 NULL，`LIST` 里的 NULL 也一样。palette
  无法定义的结果（未知的名字、没有值的色差）同样是 NULL —— 绝不会是 `NaN`，也绝不会是 0。
- **非法参数让语句失败。** 未知的 space、mode、metric、白点或格式名，列表长度不对，下标越界，都会
  报错，错误信息以函数名开头，例如 `plt_convert: unknown color space 'nope'`。
- **名字会规范化。** space、mode、metric、白点与格式名大小写不敏感，连字符与空格视作下划线：
  `'OKLCH'`、`'ok-lch'` 与 `' ok lch '` 都能用。

```sql {"type":"duckfn","show":"table","colors":"normalized"}
SELECT plt_to_hex(plt_convert(plt_srgb(1, 0.5, 0), 'OK-LCH')) AS normalized;
```

## 各类函数在哪里

| 页面 | 内容 |
| --- | --- |
| [颜色与空间](./colors/spaces.md) | 颜色值、它的读取函数、alpha，以及每个空间一个的构造器（44 个） |
| [转换与夹紧](./colors/conversion.md) | `plt_convert`、`plt_convert_clamped`、`plt_clamp`、`plt_is_within_bounds`、`plt_equals` |
| [操作](./operations/manipulation.md) | 混合、变亮、变暗、提亮、灰度、tint 与 shade、饱和度、色相与预乘 alpha |
| [混合与合成](./operations/blending.md) | 11 种混合模式、6 个 Porter-Duff 算子、自定义 OpenGL 因子 |
| [色差与对比度](./operations/difference.md) | 7 种色差度量、WCAG 2.1 亮度、对比度与判定条件、可读性选色、YIQ 亮度 |
| [配色方案](./operations/theory.md) | 互补、分裂互补、类似、三角、四方、单色 |
| [字符串、hex 与命名色](./formats/strings.md) | hex（含定宽）、完整的 CSS Color Level 4、更多输出格式、148 个 SVG/CSS3 名字、最接近的名字 |
| [8 位打包](./formats/packing.md) | 打包与解包 ARGB/RGBA/ABGR/BGRA 整数 |
| [白点、色适应与编码](./color-science/encoding.md) | 白点、色适应矩阵、传递函数、三原色、矩阵 |
| [自定义观察条件的 CAM16](./color-science/cam16.md) | 完整的 CAM16 属性，以及在自有观察条件下的 8 个 CAM16 空间 |
| [colorous 色板](./palettes/colorous.md) | d3 风格的渐变与分类色板 |
| [随机色](./palettes/random.md) | 从 palette 的分布里可复现地采样 |

## 在运行时查找函数

```sql {"type":"duckfn","show":"table"}
SELECT function_name, parameters, return_type, description
FROM duckdb_functions()
WHERE function_name LIKE 'plt_%'
ORDER BY function_name;
```

## 两个能避开大部分意外的好习惯

**先转换，再做需要某类分量的操作。** `plt_saturate` 需要带色相的空间，`plt_blend` 需要类 RGB 的空间，
配色方案函数需要一个带色相的颜色。错误信息会说明是哪个函数失败，并建议先做转换：

```sql {"type":"duckfn","expect":"error"}
SELECT plt_saturate(plt_srgb(1, 0, 0), 0.2);
```

**混合或比较时，在感知均匀的空间里做。** `oklab` / `oklch` 的混合与比较比 `srgb` 均匀得多；先转进去，
做完再转回来：

```sql {"type":"duckfn","show":"table","colors":"purple"}
SELECT plt_to_hex(plt_mix(plt_convert(plt_srgb(1, 0, 0), 'oklab'),
                          plt_convert(plt_srgb(0, 0, 1), 'oklab'),
                          0.5)) AS purple;
```
