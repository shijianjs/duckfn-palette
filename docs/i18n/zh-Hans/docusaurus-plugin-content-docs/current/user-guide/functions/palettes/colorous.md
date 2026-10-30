---
title: colorous 色板
sidebar_position: 1
description: 在某个位置采样 38 个 colorous 渐变，或沿一条列表均匀取样，并整份取用 10 个分类色板。
---

# colorous 色板

colorous 是 d3-scale-chromatic 的移植：知名的感知均匀渐变（viridis、plasma、spectral 等）与分类
色板（tableau10、set3、accent 等）。这里的一切都返回**不透明的 sRGB** 颜色；需要别的 space 就用
`plt_convert` 转换。

名字有两种、函数也有两套：

| 种类 | 数量 | 函数 |
| --- | --- | --- |
| 连续渐变 | 38 | `plt_colorous_gradient`、`plt_colorous_palette` |
| 分类色板 | 10 | `plt_colorous_categorical` |

名字就是 colorous 里的小写常量名，匹配大小写不敏感，`-` 与空格等同于 `_`（`'Blue Green'` =
`'blue_green'`）。colorous 自己有的 `grey` / `gray` 两种拼写都能用。

## 渐变

### `plt_colorous_gradient(name, t)`

**签名**：`plt_colorous_gradient(name VARCHAR, t DOUBLE) -> plt_color`

从渐变中取 `[0, 1]` 位置 `t` 处的一个颜色。超出范围的位置会被夹紧，所以 `-1` 得到起点、`2` 得到
终点，而不是报错。

```sql {"type":"duckfn","show":"table"}
SELECT plt_to_hex(plt_colorous_gradient('viridis', 0)) AS start,
       plt_to_hex(plt_colorous_gradient('viridis', 0.5)) AS middle,
       plt_to_hex(plt_colorous_gradient('viridis', 1)) AS end;
```

```sql {"type":"duckfn","show":"table"}
SELECT t, plt_to_hex(plt_colorous_gradient('spectral', t)) AS color
FROM (VALUES (0.0), (0.25), (0.5), (0.75), (1.0)) v(t);
```

`t` 是连续的，所以它可以当任意数值列的色阶：

```sql {"type":"duckfn","show":"table"}
SELECT value, plt_to_hex(plt_colorous_gradient('plasma', value / 100.0)) AS color
FROM (VALUES (10), (45), (80)) v(value);
```

### `plt_colorous_palette(name, n)`

**签名**：`plt_colorous_palette(name VARCHAR, n BIGINT) -> plt_color[]`

沿渐变从起点到终点均匀取样的 `n` 个颜色。`n` 至少为 1；0 或负数会报错。

```sql {"type":"duckfn","show":"table"}
SELECT to_json(list_transform(plt_colorous_palette('viridis', 5), lambda x: plt_to_hex(x))) AS five,
       length(plt_colorous_palette('viridis', 5)) AS count;
```

这是用连续渐变搭分类色阶的常见做法 —— 有多少个类别就取多少个颜色：

```sql {"type":"duckfn","show":"table"}
SELECT i, plt_to_hex(plt_colorous_palette('turbo', 4)[i]) AS color
FROM range(1, 5) t(i);
```

### `plt_colorous_palettes()`

**签名**：`plt_colorous_palettes() -> VARCHAR[]`

全部 38 个渐变名 —— 多色相顺序渐变（`turbo`、`viridis`、`inferno`、`magma`、`plasma`、
`cividis`、`warm`、`cool`、`cubehelix`、`blue_green`、`blue_purple`、`green_blue`、`orange_red`、
`purple_blue_green`、`purple_blue`、`purple_red`、`red_purple`、`yellow_green_blue`、`yellow_green`、
`yellow_orange_brown`、`yellow_orange_red`）、单色相（`blues`、`greens`、`greys`、`oranges`、
`purples`、`reds`）、发散（`brown_green`、`pink_green`、`purple_green`、`purple_orange`、
`red_blue`、`red_grey`、`red_yellow_blue`、`red_yellow_green`、`spectral`）与循环（`rainbow`、
`sinebow`）。

```sql {"type":"duckfn","show":"table"}
SELECT list_count(plt_colorous_palettes()) AS count,
       plt_colorous_palettes() AS gradients;
```

## 分类色板

### `plt_colorous_categorical(name)`

**签名**：`plt_colorous_categorical(name VARCHAR) -> plt_color[]`

整份分类色板。长度由色板本身决定：`category10` 与 `tableau10` 是 10 个，`paired` 与 `set3`
是 12 个，`pastel1` 与 `set1` 是 9 个，`accent`、`dark2`、`pastel2` 与 `set2` 是 8 个：

```sql {"type":"duckfn","show":"table"}
SELECT to_json(list_transform(plt_colorous_categorical('tableau10'), lambda x: plt_to_hex(x))) AS tableau10;
```

```sql {"type":"duckfn","show":"table"}
SELECT name, length(plt_colorous_categorical(name)) AS colors
FROM unnest(plt_colorous_categoricals()) t(name);
```

### `plt_colorous_categoricals()`

**签名**：`plt_colorous_categoricals() -> VARCHAR[]`

10 个分类色板名：`category10`、`accent`、`dark2`、`paired`、`pastel1`、`pastel2`、`set1`、
`set2`、`set3`、`tableau10`。

```sql {"type":"duckfn","show":"table"}
SELECT plt_colorous_categoricals() AS palettes;
```

## 错误

未知的名字会让查询失败，错误信息会指向列举函数：

```sql {"type":"duckfn","expect":"error"}
SELECT plt_colorous_gradient('viridus', 0.5);
```

```sql {"type":"duckfn","expect":"error"}
SELECT plt_colorous_categorical('tableau20');
```

```sql {"type":"duckfn","expect":"error"}
SELECT plt_colorous_palette('viridis', 0);
```
