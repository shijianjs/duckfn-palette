---
title: 简介
sidebar_position: 1
slug: /intro
description: duckfn_palette 给 DuckDB 加了什么，以及怎么开始用。
---

# 简介

`duckfn_palette` 是一个 [DuckDB 可加载扩展](https://duckdb.org/docs/stable/extensions/overview)，把三个
Rust 颜色库包装成 SQL 函数：[palette](https://crates.io/crates/palette) 提供 44 个颜色空间、转换、混合、
色差与对比度、配色方案、白点、CAM16 与 8 位打包；[csscolorparser](https://crates.io/crates/csscolorparser)
负责 CSS Color Level 4 字符串；[colorous](https://crates.io/crates/colorous) 提供 d3-scale-chromatic 的
渐变与分类色板。三者一起注册了 `plt_` 前缀下的 160 个函数，加载之后它们和 DuckDB 自带的函数没有区别。
你不需要知道它是怎么写出来的，也不用编译任何东西 —— CLI、Python / R 会话与浏览器里的构建接受的都同一份
`.duckdb_extension` 文件。

颜色就是一个 `STRUCT`，所以它能待在列里、当值传来传去，也能直接比较：

```text
STRUCT(space VARCHAR, c1 DOUBLE, c2 DOUBLE, c3 DOUBLE, alpha DOUBLE)
```

## 装上它

```sql
INSTALL duckfn_palette FROM community;
LOAD duckfn_palette;
```

然后在任意查询里调用：

```sql {"type":"duckfn","show":"table"}
SELECT plt_to_hex(plt_convert(plt_srgb(1, 0.5, 0), 'oklch')) AS as_oklch,
       plt_difference(plt_named('red'), plt_named('blue'), 'ciede2000') AS delta_e;
```

上面这个块就在你的浏览器里真跑：站点从项目的最新 Release 预加载了这个扩展，所以这里不用写 `LOAD`。
其他获取文件的方式见[安装](./installation.md)。

## 接下来去哪

- [安装](./installation.md) —— 社区仓、Release 文件，或者本地自己构建的产物。
- [函数](./functions/overview.md) —— 全部函数，按用途分组，各配一个能就地跑的例子。
- [开发指南](../development/quick-start.md) —— 从源码构建这个扩展。那是给改这个仓库的人看的；用扩展
  本身不需要它。
