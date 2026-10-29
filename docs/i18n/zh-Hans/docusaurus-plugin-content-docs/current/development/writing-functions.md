---
title: 编写函数
sidebar_position: 3
description: plt_ 函数是怎么写的、duckfn 对入参与返回值的规则，以及新增函数时该抄哪一段。
---

# 编写函数

一个注册进 DuckDB 的函数 = 「一个普通 Rust 函数 + 一个属性宏」。宏负责生成 FFI 包装层、读参数列、写
结果列、提交注册项；函数体里只有你的逻辑。

从一个普通函数到可调用的 SQL 函数：

```mermaid
flowchart LR
    fn["普通 Rust 函数"] --> macro["duck 属性宏"]
    macro --> wrapper["FFI 包装层与<br/>注册项"]
    wrapper --> cdylib["扩展二进制<br/>cdylib"]
    cdylib --> load["在 DuckDB 里 LOAD"]
```

## 本仓在用的形状

| 形状 | 例子 | 语义 |
| --- | --- | --- |
| `-> T` | `plt_srgb(c1, c2, c3) -> PltColor` | 朴素值，永不为 NULL。 |
| `-> DuckOptionResult<T>` | `plt_convert(color, space)` | 可空 + 可报错：`Ok(None)` 是 SQL `NULL`，`Err` 让查询失败。 |
| `-> Vec<T>` | `plt_colorous_palette(name, n)` | SQL `LIST`，每个输入行一个列表。 |
| `-> DuckOptionResult<Vec<T>>` | `plt_rgb_primaries(space)` | 既可能是 NULL、也可能报错的列表。 |
| `overloads_name` | `plt_pack`、`plt_difference` | 多个 Rust 函数注册在同一个 SQL 名字下。 |

全是标量函数 —— 这个扩展里没有聚合、表函数或 `COPY` 处理器。`src/extension/functions/` 与 palette 的
模块一一对应，所以新函数放进覆盖它那个上游 trait 的文件里。

## 标量：三种返回形状

宏按返回类型生成不同的代码：

| 签名 | 语义 |
| --- | --- |
| `-> T` | 朴素值，永不为 NULL。 |
| `-> DuckOptionResult<T>` | 可空 + 可报错：`Ok(None)` 落成 SQL `NULL`，`Err` 让整条查询失败。 |
| `-> Option<T>` | 可空，但报不了错。 |

```rust
use duckfn::{DuckOptionResult, duck_scalar_function};

#[duck_scalar_function(
    description = "Convert a color to another space without gamut clamping (palette's unclamped conversion)",
    example = "SELECT plt_components(plt_convert(plt_srgb(1, 0, 0), 'lab'))"
)]
fn plt_convert(color: PltColor, space: String) -> DuckOptionResult<PltColor> {
    Ok(Some(color.convert("plt_convert", &space)?))
}
```

三态契约在每个文件里都是同一套：NULL 参数给 NULL（duckfn 在函数体执行前就短路掉了）、palette 算不出的
给 `Ok(None)`、被拒绝的参数给 `Err`，消息以函数名开头。

### 传入 NULL 时会发生什么

参数是否可空由 **参数类型** 决定：

- **`space: String`** —— NULL 行被 duckfn 的参数读取层短路成 SQL `NULL`，函数体根本不会执行到那一行。
  多数情况下你要的就是这个。
- **`space: Option<String>`** —— NULL 以 `None` 进函数体，语义由你决定（返回 NULL、换成默认值……）。

### 错误与 panic

遇到处理不了的值就 `Err(duck_error("…"))`，整条查询会带着你的消息失败。函数体里的 `panic!` 会被捕获、
作为 DuckDB 错误上报，而不是展开着穿过 FFI 边界 —— palette 自己的 panic 正是这样变成查询错误的：

```rust
// palette 遇到奇异矩阵会 panic；扩展把它变成一条错误
plt_matrix_inverse([0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]);
// -> The given matrix is not invertible
```

错误信息是给用户看的：用英文写，并带上函数名做前缀，这样单独一行报错也读得懂。

## 自定义类型

`types/color.rs` 定义了 `PltColor`，也就是每个函数都收发的那个 `plt_color` STRUCT：

```rust
#[derive(DuckStruct)]
#[duck(sql_name = "plt_color", create_type = true)]
pub struct PltColor {
    pub space: String,
    pub c1: f64,
    pub c2: f64,
    pub c3: f64,
    pub alpha: f64,
}
```

44 个空间共用一个 STRUCT，接口面才小得下来：空间名跟着值走，所以 `plt_convert` 不需要为每个空间配一对
函数，颜色列也能靠 `SELECT *` 原样往返。代价是分量只有位置名（`c1`、`c2`、`c3`），这正是
`plt_component(color, index)` 与 `plt_components(color)` 存在的理由。

`cam16.rs` 又加了两个：`plt_cam16_params`（五个观察条件加去色散）与 `plt_cam16_full`（六个 CAM16 属性）。

## 新增一个函数

1. **挑属性。** 这个扩展只需要 `duck_scalar_function`；duckfn 另有表函数、`COPY`、cast、SQL 宏、替换
   扫描，各有各的宏和各自接受的参数。参考
   [duckfn 用户指南](https://shijianjs.github.io/duckfn/zh-Hans/) —— 同一份指南也随 crate 发布，就在
   `Cargo.toml` 所钉版本在 cargo registry 里的解包目录中。
2. **从对应 palette 模块的那个文件里抄最接近的邻居**，不要凭印象自创签名。
3. **挂进模块树**：在 `src/extension/functions/mod.rs` 里加一行 `mod …;`（需要类型时再加到
   `src/extension/types/mod.rs`）。crate root 不用动 —— 只是 `src/wasm_lib.rs` 与它们镜像，所以新模块
   两边都要出现。
4. **写文档元数据**：属性上的 `description`、`comment`、`example` / `examples`：

   ```rust
   #[duck_scalar_function(
       description = "One line for the function table of the community-extension page",
       comment = "The detail that does not fit the one-liner",
       example = "SELECT plt_to_hex(plt_convert(plt_srgb(1, 0.5, 0), 'oklch'))"
   )]
   ```

   DuckDB 的 C 扩展 API 没有设置描述与示例的接口，所以这段文本是社区扩展页 `Added Functions` 表的唯一
   来源。`just docs_csv` 会把它导出到 `target/function_descriptions.csv`（见[社区扩展](./community-extension.md)）。
   文案用英文写 —— 它会被原样贴到那个页面上。
5. **补测试**（见[测试](./testing.md)），然后跑 `just lint`。

### 命名

每个 SQL 名字都带 `plt_` 前缀，前缀之后的部分要能读出这个函数做什么 —— 构造函数用空间名（`plt_oklch`）、
操作用 palette 的 trait 名（`plt_premultiply`）、清单函数用被列举的东西（`plt_blend_modes`）。这个名字是
用户要敲的，所以别写成 `plt_palette_oklch` 这类堆砌。

属性宏默认拿 **Rust 函数名**当注册名，函数因此就叫 `plt_convert`。同一个名字下需要多个签名（参数类型或
个数不同）时，用 `overloads_name = "…"` 把它们并成一个函数集，而不是各自注册 —— `plt_pack`、
`plt_unpack`、`plt_difference`、`plt_chromatic_adapt`、`plt_cam16_params` 各自接受两种形状，就是这么做的。

宏还会为每个签名生成 `SQL_NAME` 常量。同一个名字要在多处出现时（错误信息前缀、日志、提示），读那个
常量，别再抄一份字面量；代价是这类函数得写成 `pub(super)`，因为生成的模块沿用函数的可见性。

### 有副作用的函数

有副作用（打印、写文件、起进程）或非确定性的函数必须声明 `volatile = true`。否则优化器会在常量折叠阶段
执行它一次、折进外层表达式后又一次，`EXPLAIN` 也会真的把它跑一遍。`plt_random_color` 与
`plt_random_colors` 特意带一个种子参数，让结果可复现 —— 不带种子的随机函数就必须写 `volatile = true`。

### 配置参数

配置类参数（`DuckLazy<T>`）只在函数内部解析一次，逐行解析不会出现在 profile 里。这个扩展没有用它：这里
的每个选项都是一个朴素的 `VARCHAR` 名字，由对应的 `types/` 文件里一张小表解析。那套写法在 duckfn 指南里
有，[duckfn_quantstats](https://github.com/shijianjs/duckfn-quantstats) 里也到处都是。
