---
title: 字符串、hex 与命名色
sidebar_position: 1
description: 解析与格式化 hex，处理完整的 CSS Color Level 4 语法，查询 148 个 CSS/SVG 名字，并把颜色还原成它的关键字。
---

# 字符串、hex 与命名色

有两个覆盖面不同的解析器。**Hex**（`plt_from_hex`、`plt_to_hex`）是 palette 自己的读写器：紧凑、
严格，不认识颜色名。**CSS**（`plt_css_parse`、`plt_css_serialize`）走 csscolorparser，能理解完整的
CSS Color Level 4 语法，包括 `lab()`、`oklch()` 与 `color(display-p3 …)`。

两个解析器产出的都是 **sRGB** 颜色，因为这两种格式编码的都是 sRGB。字符串永远不会直接解析成
`oklch` 值：先解析，想要别的 space 再用 `plt_convert`。

## Hex

### `plt_from_hex(text)`

**签名**：`plt_from_hex(text VARCHAR) -> plt_color`

解析 `#rgb`、`#rgba`、`#rrggbb` 或 `#rrggbbaa`；开头的 `#` 可以省略。短格式把每一位重复展开
（`#f80` 就是 `#ff8800`），4 位或 8 位的值带 alpha。不是 hex 颜色的文本返回 **NULL** 而不是报错
—— 所以它能和 `WHERE`、`COALESCE` 一起用：

```sql {"type":"duckfn","show":"table"}
SELECT plt_to_hex(plt_from_hex('#f80')) AS short,
       plt_to_hex(plt_from_hex('#ff800080')) AS with_alpha,
       plt_to_hex(plt_from_hex('ff8000')) AS without_hash,
       plt_from_hex('orange') IS NULL AS unknown_is_null;
```

### `plt_to_hex(color)`

**签名**：`plt_to_hex(color plt_color) -> VARCHAR`

先转成 sRGB，再把每个通道量化到 8 位并写成 `#rrggbb`；alpha 不恰好是 1 时改写成 `#rrggbbaa`。
超出色域的分量会被夹紧，所以结果永远是一个合法的 hex 字符串：

```sql {"type":"duckfn","show":"table"}
SELECT plt_to_hex(plt_srgb(1, 0.5, 0)) AS orange,
       plt_to_hex(plt_color('srgb', 1, 0.5, 0, 0.5)) AS translucent,
       plt_to_hex(plt_oklch(0.7, 0.1, 120)) AS from_oklch;
```

## CSS 语法

### `plt_css_parse(text)`

**签名**：`plt_css_parse(text VARCHAR) -> plt_color`

把任意 CSS Color Level 4 颜色解析成带 alpha 的 sRGB 颜色。这涵盖命名色、所有 hex 形式、`rgb()`、
`rgba()`、`hsl()`、`hsla()`、`hwb()`、`lab()`、`lch()`、`oklab()`、`oklch()`，以及带常见颜色空间的
`color(...)`。分隔符可以是逗号或空格，alpha 可以是数字或百分比。

无法解析的文本返回 **NULL**：

```sql {"type":"duckfn","show":"table"}
SELECT plt_to_hex(plt_css_parse('hsl(120 100% 50% / 0.5)')) AS hsl,
       plt_to_hex(plt_css_parse('rgb(255, 128, 0)')) AS rgb,
       plt_to_hex(plt_css_parse('oklch(0.7 0.15 250)')) AS oklch,
       plt_css_parse('not a color') IS NULL AS invalid_is_null;
```

### `plt_css_is_valid(text)`

**签名**：`plt_css_is_valid(text VARCHAR) -> BOOLEAN`

`plt_css_parse` 会不会成功。可以在解析前用它校验一列，或者用它把无效的行留在报表里可见：

```sql {"type":"duckfn","show":"table"}
SELECT plt_css_is_valid('#ff0000') AS hex_ok,
       plt_css_is_valid('rgb(300 0 0)') AS clampable,
       plt_css_is_valid('rgb(nope)') AS broken;
```

### `plt_css_serialize(color, format)`

**签名**：`plt_css_serialize(color plt_color, format VARCHAR) -> VARCHAR`

把颜色写成 CSS 文本，写之前先转成 sRGB。八种格式就是 csscolorparser 提供的八个函数：`hex`、`rgb`、
`hsl`、`hwb`、`lab`、`lch`、`oklab`、`oklch`。

```sql {"type":"duckfn","show":"table"}
SELECT plt_css_formats() AS formats;
```

```sql {"type":"duckfn","show":"table"}
SELECT plt_css_serialize(plt_srgb(1, 0.5, 0), 'hex') AS hex,
       plt_css_serialize(plt_srgb(1, 0.5, 0), 'rgb') AS rgb,
       plt_css_serialize(plt_srgb(1, 0.5, 0), 'oklch') AS oklch,
       plt_css_serialize(plt_srgb(1, 0.5, 0), 'lab') AS lab;
```

走一趟 CSS 文本的完整往返仍留在 sRGB，并且保留 alpha：

```sql {"type":"duckfn","show":"table"}
SELECT plt_css_parse(plt_css_serialize(c, 'oklch')) AS reparsed,
       plt_to_hex(plt_css_parse(plt_css_serialize(c, 'oklch'))) AS hex
FROM (SELECT plt_color('srgb', 0.2, 0.4, 0.6, 0.5) AS c);
```

未知的格式让整条语句失败：

```sql {"type":"duckfn","expect":"error"}
SELECT plt_css_serialize(plt_srgb(1, 0, 0), 'hsv');
```

### `plt_css_name(color)`

**签名**：`plt_css_name(color plt_color) -> VARCHAR`

sRGB 字节与该颜色完全一致的 CSS 关键字；没有关键字匹配时是 NULL。这是精确匹配查找，不是找最接近的
名字 —— 大多数颜色根本没有名字：

```sql {"type":"duckfn","show":"table"}
SELECT plt_css_name(plt_srgb(1, 0, 0)) AS red,
       plt_css_name(plt_named('rebeccapurple')) AS purple,
       plt_css_name(plt_srgb(0.2, 0.4, 0.6)) IS NULL AS unnamed;
```

### `plt_css_formats()`

**签名**：`plt_css_formats() -> VARCHAR[]`

`plt_css_serialize` 接受的八个格式名。

## 命名色

### `plt_named(name)`

**签名**：`plt_named(name VARCHAR) -> plt_color`

palette 的 148 个 SVG/CSS3 关键字颜色之一，作为不透明的 sRGB 颜色返回。匹配不区分大小写。未知的
名字返回 **NULL**：

```sql {"type":"duckfn","show":"table"}
SELECT plt_to_hex(plt_named('rebeccapurple')) AS purple,
       plt_to_hex(plt_named('Gold')) AS gold,
       plt_named('notacolor') IS NULL AS unknown_is_null;
```

### `plt_named_names()`

**签名**：`plt_named_names() -> VARCHAR[]`

全部关键字名字，按字母顺序排列。用它来建一张参考表，不必把清单抄进 SQL：

```sql {"type":"duckfn","show":"table"}
SELECT list_count(plt_named_names()) AS count,
       plt_named_names() AS names;
```

```sql {"type":"duckfn","show":"table"}
SELECT name, plt_to_hex(plt_named(name)) AS hex
FROM unnest(plt_named_names()) t(name)
WHERE name IN ('red', 'green', 'blue', 'rebeccapurple');
```

## 该用哪个函数？

| 输入或输出 | 函数 |
| --- | --- |
| 来自数据库列的 hex 字符串 | `plt_from_hex` |
| 来自样式表或 API 的 CSS 颜色 | `plt_css_parse` |
| 用于展示的短 hex 字符串 | `plt_to_hex` |
| 某个特定的 CSS 函数，例如 `oklch(...)` | `plt_css_serialize` |
| 148 个关键字颜色之一 | `plt_named` / `plt_named_names` |
| 某个颜色的关键字名字 | `plt_css_name` |
| 文本是不是合法的 CSS 颜色 | `plt_css_is_valid` |
