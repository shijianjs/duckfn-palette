# AGENTS.md —— src/ 的代码约定

这份文件是给改 `src/` 的人（含 AI agent）看的。整体流程（立项、上游盘点、映射决策、文档与发布）
见 [`../AGENTS/wrapping-a-rust-library.md`](../AGENTS/wrapping-a-rust-library.md)；duckfn 的 API
在哪查、查不到怎么办，见根 [`../AGENTS.md`](../AGENTS.md)。本文件只管「落到代码里怎么写」。

## 目录：与上游模块树同构

包装类项目的目录不是按「标量/聚合」分，而是**按上游模块分** —— 接手的人拿着上游源码就能找到
对应文件：

```text
src/extension/functions/
    mod.rs                    对应表（上游模块 ↔ 本仓库文件）+ 共享 helper
    <上游模块 A>.rs / <上游模块 A>/
    <上游模块 B>.rs / <上游模块 B>/
```

`functions/mod.rs` 的头注释放三样东西，中英双语：

1. 对应表：上游每个模块 → 本仓库哪个文件；
2. 「不包装清单 + 理由」（全量包装项目必填）；
3. 三态契约（NULL / 上游哨兵 / 调用错误分别怎么走）。

文件内的组织：一个函数组一个文件，文件头注释写清「这一类是什么、为什么是这个形状、NULL 怎么处理」。
函数本身按上游顺序排列，`///` 文档里给两三条 `SELECT …` 示例。

## 形状：多列输入优先做成聚合

| 输入 | 形状 |
| --- | --- |
| 一列进、一个值出 | `#[duck_aggregate_function(auto_collect = true)]` + `Vec<T>` 参数 |
| 两列按下标配对 | 同上，再加一个 `Vec<U>` 参数（同一行两个值配对） |
| 没有下标对应的两列 | 不要收两个列表；让用户给 label/value，聚合按分组建状态 |
| 逐行求值 | `#[duck_scalar_function]` |
| 返回一串值 | 标量，返回 `Vec<T>`（即 `LIST(...)`） |

`SELECT my_stat(x) FROM t GROUP BY g` 是数据库用户的默认写法；做成 `Vec` 参数 + 标量会迫使每个
调用点先 `list(x)`，而且 DuckDB 的 `list()` **保留 NULL 元素**，与聚合跳过 NULL 的语义不一致。

## 标量：三种返回形状

| 签名 | 语义 |
| --- | --- |
| `-> T` | 朴素值，永不为 NULL |
| `-> DuckOptionResult<T>` | 可空 + 可报错：`Ok(None)` 是 SQL NULL，`Err` 让整条查询失败 |
| `-> Option<T>` | 可空但报不了错 |

包装类项目基本只用第二种：上游的「算不出」要能折成 NULL，参数非法要能报错，两种出口都得有。

入参可空性是另一条轴：参数写 `T` 时 NULL 行被读取层短路（函数体不执行），写 `Option<T>` 时 NULL
以 `None` 进函数体、语义由你定。标量与聚合同一套规则。

## 聚合：用 `auto_collect`，不要手写状态

```rust
#[duck_aggregate_function(auto_collect = true)]
fn my_stat(values: Vec<f64>) -> DuckOptionResult<f64> {
    nan_to_null(values.mean())
}
```

被注解函数**就是** finalize 处理器：`Vec<T>` 是逐行收集的列，`DuckFirst<T>` 是每查询解析一次的
常量（例如采样个数 `k`），返回值沿用标量规则。状态、`simple_combine`、`result` 全由宏生成。

- **不要**写状态结构体、marker 类型或 `DuckAggregateState` impl，也不要写 `macro_rules!`。
- 非 `Option` 列的 NULL 整行不进收集（配对场景因此免费）；空组配上非空 `DuckFirst<T>` 直接回
  NULL，不调函数。
- 真需要自定义状态形状时才手写回退，形状参考 duckfn 的 `ScalarFunctionAdapter`。

## 三态契约：一处定义，处处套用

| 情况 | 表现 | 写法 |
| --- | --- | --- |
| 输入 NULL | 默认短路成 NULL | 参数用 `T` |
| 上游哨兵（NAN / 空 `Option` / `Err`） | NULL | 走共享 helper，例如 `functions/mod.rs::nan_to_null` |
| 参数存在但非法 | 查询报错 | `Err(duck_error("my_fn: …"))` |
| 结果是 ±∞ | 原样返回 | 不要折成 NULL |

- 哨兵值永远不能出现在结果列里。所有包装都走同一个 helper，不要每个文件自己写 `if x.is_nan()`。
- 错误信息用英文、以函数名开头（`"my_normal_pdf: standard deviation must be positive, got -1"`），
  用户报错时能自解释。参数校验的 helper（概率范围、正数、区间顺序）放子模块 `mod.rs`，别复制。
- 不要用 `panic!` 做参数校验：panic 会被 duckfn 捕获成 DuckDB 错误，但消息没有上下文。

## 副作用与非确定性：一律 `volatile = true`

会碰外面世界的标量函数（打印到 stdout、写文件、起进程、发请求）和结果不稳定的函数（采样、随机数、
当前时间戳）都必须标 `volatile = true`：

```rust
#[duck_scalar_function(volatile = true)]
fn my_write_file(content: String, ext: String) -> DuckOptionResult<String> { … }
```

不标的后果（[duckfn_open](https://github.com/shijianjs/duckfn-open) 实测）：

- 常量参数的调用在**优化期**先求值一次、折成常量；折出来的值被外层表达式吃掉时，执行期会**再求值
  一次** —— `SELECT length(my_write_file('a','txt'))` 落**两个**文件、`SELECT length(my_print(...))`
  打**两遍**；
- `EXPLAIN SELECT my_write_file('a','txt')` 会**真的写一个文件**：只做规划、不打算执行的查询不该有
  副作用。

标上之后语义是「一次调用 = 一行，不多不少」，常量参数也逐行（`SELECT my_print('x') FROM t` 有几行
打几行）。因此：

- **文档 / README / `description.yml` 里不要承诺「常量只跑一次」**；
- **纯函数不要加**，白丢优化器的复用与折叠。判断标准：同样输入调用两次结果可能不同吗？会碰外面世界吗？
- 返回值带上入参（文本进文本出、内容进路径出），副作用之外还有值可用，用户能 `length(...)` 折叠掉；
- 写 stdout 记得 flush：Rust 的 stdout 按行缓冲，不带换行的输出会卡在缓冲区里，看起来像没打；
- 不要承诺逐行副作用的顺序：并行扫描下它与表顺序无关。

测试怎么断言这类函数见 [`../test/AGENTS.md`](../test/AGENTS.md) 的「副作用函数怎么测」。

## 类型映射

照上游原文透出，不要一律 `DOUBLE`：`u64 → UBIGINT`、`i64 → BIGINT`、`u32 → UINTEGER`、
`f64 → DOUBLE`、`bool → BOOLEAN`、`Vec<f64> → LIST(DOUBLE)`。唯一允许的简化是泛型参数
（`impl Into<f64>`）只声明实数入口。

需要自定义类型（STRUCT / ENUM / `list<struct>` 行类型 / `DuckLazy` 配置）时放 `types/`，并在加载
时注册；不需要就把整个目录删掉（连同 `extension/mod.rs` 里的 `mod types;`）。

## 命名

- 前缀统一（模板 `my_`，statrs 项目 `sr_`），后面直接沿用上游名字：`Statistics::mean` → `sr_mean`。
- 上游名字在 SQL 里读不通时改名，并把映射写进 `description` 与文档（`inverse_cdf` → `quantile`）。
- 属性宏默认拿 **Rust 函数名**当注册名；同名多签名用 `overloads_name` 合并。
- 宏为每个签名生成 `SQL_NAME` 常量：错误前缀、日志里读它，不要再抄字面量（代价是函数要写成
  `pub(super)`）。

## 文档元数据（社区扩展页的唯一来源）

每个注册函数都写 `description`（一句），必要时加 `comment`（细节）与 `example`（一条完整 SQL）：

```rust
#[duck_scalar_function(
    description = "Normal (Gaussian) cumulative distribution function P(X <= x)",
    comment = "A probability outside [0, 1] is a query error rather than a clamped endpoint",
    example = "SELECT my_normal_cdf(1.96, 0.0, 1.0)"
)]
```

DuckDB 的 C 扩展 API 没有设置描述的接口，这段文本是社区扩展页表格的唯一来源。**一律英文**，
会被原样贴到页面上；多条示例用 `examples`，与 `example` 互斥。`just docs_csv` 导出，
`cargo run --bin duckfn -- function_descriptions --all` 看谁还没写。

## 依赖

- 只引真正需要的 crate，并在 `Cargo.toml` 里用一句注释写清「它负责什么、为什么选它」。
- 上游 crate 的 feature 收窄到实际用到的（默认 features 常常带一堆用不上的重量级依赖）。
- 平台相关的依赖挂到 `[target.'cfg(...)'.dependencies]`，别让别的目标替它付编译成本。
- 通用逻辑（平台差异、临时文件、编码、哈希、日期算术）优先用现成 crate 或 std，规则见根
  [`AGENTS.md`](../AGENTS.md) 的「尽量用成熟三方库实现」。

## 铁律

duckfn 的属性名、参数、允许的返回形状**只从本机源码/文档确认**（路径见根 `AGENTS.md`）。
查不到就停下来问，不要凭记忆编 —— 编出来的宏会一路编到编译不过为止，返工成本远高于问一句。
