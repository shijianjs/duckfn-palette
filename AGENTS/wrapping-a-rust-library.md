# 把一个 Rust 库映射成 DuckDB 扩展

适用场景：上游 crate 本身就是全部价值（统计、解析、编码、几何、时间序列……），本仓库只负责给它
一层 SQL 出口。模板自带的 `my_greet` / `my_sum` 演示的是「怎么写一个函数」；包装类项目真正难的
是「怎么把一整棵树搬过来，并且证明没搬漏」。

参考实现：[duckfn_statrs](https://github.com/shijianjs/duckfn-statrs) —— 把 statrs 的
34 个聚合 + 509 个标量（543 个注册名，含 `Modulus` 的 5 个整数/浮点 overload）全量搬上 SQL。
本文件是那次工作的反推流程，坑也是它踩出来的。副作用与 `volatile` 那一节来自
[duckfn_open](https://github.com/shijianjs/duckfn-open)（打印、写文件、起进程这类会碰外面世界的函数），
文档里展示图像那一节来自 [duckfn_kuva](https://github.com/shijianjs/duckfn-kuva)（返回值本身就是 SVG
的画图类扩展）。

工作量大致是 **代码 2 : 测试 3 : 文档 2 : 发布 1**。测试比代码多不是笔误：一个几百函数的扩展，
函数体几乎都是「一行转发」，真正的风险全在语义（NULL、边界、并行）与文档缺口上。

> 相关的两份文件夹级约定：[`../src/AGENTS.md`](../src/AGENTS.md)（写代码）、
> [`../test/AGENTS.md`](../test/AGENTS.md)（写测试）。本文件只讲流程与决策，不重复那里的规则。

## 0. 立项：先定范围，再写第一行代码

| 选择 | 你要额外做的事 | 代价 |
| --- | --- | --- |
| **全量包装**（上游能力不允许缺失） | 维护一张「上游模块 ↔ 本仓库文件」对应表；用机器核对差集；每个上游 `pub` 面要么有 SQL 出口，要么有一条「为什么不包」的理由 | 函数可能上百/上千；漏一个不会报错，只会长期没人发现 |
| **精选子集** | 在对应表旁边写「不包装清单 + 理由」 | 用户会反复问「为什么没有 X」，理由要能自洽 |

选了全量，就把这条规则写进 `src/extension/functions/mod.rs` 的**头注释**，让下一个接手的人（和下一次
会话的 AI）第一眼看到它 —— 口头约定活不过一次上下文清空。

立项时顺手确认三件事：

1. **上游 MSRV**：读上游 `Cargo.toml` 的 `rust-version`。它可能高于模板默认下限（statrs 0.19 要
   Rust 1.89，模板原本是 1.86）。等 `cargo build` 报错才发现，会连带怀疑整条工具链。
2. **上游能不能编到 wasm**：`just build_wasm` 与文档站的可运行 SQL 块都依赖它。上游用了
   `std::fs`、线程、系统库的话，要么避开那些 API，要么放弃 wasm 路线（那就别启用 `docs/` 的
   可运行块）。
3. **许可证与出处**：README 与 `community-extension/description.yml` 都要写清上游是谁、什么协议。

## 1. 盘点上游 API（读本机源码，不读网页）

上游资料读本机 cargo registry 里解包的那份，版本与 `Cargo.toml` 严格对应：

```bash
ls -d ~/.cargo/registry/src/*/<上游 crate>-*/
```

```powershell
Get-ChildItem "$env:CARGO_HOME\registry\src\*\*-<上游 crate>-*" -Directory |
    Select-Object -ExpandProperty FullName
```

然后把「能力清单」拉出来，别靠读文档目录猜：

```bash
# 公开面：函数、常量、trait、类型
grep -rn "pub fn \|pub const \|pub trait \|pub struct \|pub enum " <crate>/src --include=*.rs

# 测试素材：doc example（doctest）与文件尾部的 #[cfg(test)] mod tests
grep -rn "#\[cfg(test)\]" -A80 <crate>/src --include=*.rs
```

两件事同时完成：

- **能力清单** → 后面逐条核对「有没有 SQL 出口」。清单落成临时文件放 `target/`，不要提交。
- **测试素材** → 上游 doc example 与 `mod tests` 里的 `assert` 是期望值的**唯一可信来源**
  （见 `../test/AGENTS.md`）。盘点阶段就把「哪个函数有上游用例、用例几条」记下来，
  写测试时按它分配断言数。

上游是 trait 方法族（`Statistics::mean`、`ContinuousCDF::cdf`……）时，清单按 trait 分组，
一个 trait 往往就是本仓库的一个文件、一批同前缀函数。

## 2. 形状映射：先定形状，再写函数

同一个上游函数可以有多种 SQL 形状，选错会让每个调用点都别扭。按下面这张表决定：

| 上游形状 | SQL 形状 | 理由 / 注意 |
| --- | --- | --- |
| 一列进、一个标量出（统计量） | `#[duck_aggregate_function(auto_collect = true)]`，`Vec<T>` 参数 | `SELECT sr_mean(x) FROM t GROUP BY g` 是数据库用户本来就会写的形状 |
| 两列、按下标严格配对（协方差） | 两个逐行参数 `f(x, y)` 的聚合 | 任一列 NULL 整行跳过；上游「不等长就 panic」的 case 在配对形状下构造不出来 |
| 两列、没有下标对应 | 收 label/value（分组键列 + 数值列） | 别逼用户自己对齐两个列表，聚合天然按分组建状态 |
| 逐行求值（pdf / cdf / quantile、特殊函数） | 标量 | 参数来自列时同样能用 |
| 零参常量（π、精度阈值） | 零参标量 | 不要做成表函数 |
| 返回一串值（采样、序列生成） | 返回 `Vec<T>`（即 `LIST(...)`）的标量 | 元素类型要定死：`LIST(DOUBLE)` / `LIST(UBIGINT)` |
| 构造器（`Normal::new`） | 不注册；参数摊平进每个函数 | `sr_normal_pdf(x, mean, std_dev)`，别让用户先构造对象 |
| 同一能力的 `checked_*` / 错误路径孪生 | 只包一份 | SQL 侧「算不出 → NULL」已经覆盖两条路径，包两份只是把同一函数注册两次 |
| 泛型参数（`impl Into<f64>`） | 只声明实数入口 | 一种类型一个重载会把函数表撑爆，而 DuckDB 的隐式转换已覆盖 |
| 同名、多类型重载（如 `Modulus` 的 f32/整数版） | 用 `overloads_name` 合并，或只留最通用的一份 | `duckdb_functions()` 按**签名**计数，数字会大于函数名数，别拿它当函数个数 |

### 命名

- 前缀统一（模板是 `my_`，statrs 项目是 `sr_`），前缀之后直接沿用上游名字：`Statistics::mean` →
  `sr_mean`。用户搜得到、也对得上上游文档。
- 上游名字在 SQL 里读不通时再改名，并把映射写进文档与 `description`：`inverse_cdf` → `quantile`、
  `ln_pdf` → `ln_pdf`（保留）。
- 一个上游函数拆成多个 SQL 函数时（`sr_normal_pdf` / `sr_normal_cdf` / …），命名要有规律，
  让 `duckdb_functions()` 按前缀一搜就是一整族。

### 类型映射

上游类型照原文透出，不要一律 `DOUBLE`：`u64 → UBIGINT`、`i64 → BIGINT`、`u32 → UINTEGER`、
`f64 → DOUBLE`、`bool → BOOLEAN`、`Vec<f64> → LIST(DOUBLE)`、计数向量 → `LIST(UBIGINT)`。
唯一允许的简化是泛型入口只留一份（见上表）。完整的 Rust ↔ DuckDB 映射表在 duckfn 的
`docs/docs/guide/types.md`。

### 三态契约（**先定，再逐函数套用**）

上游库通常用哨兵值表达「算不出」（NAN、空 `Option`、`Err`），SQL 有自己的一套。把这张表定死，
写进 `functions/mod.rs` 的头注释，之后每个函数都照它来，不要逐函数重新发明：

| 情况 | SQL 表现 | 怎么实现 |
| --- | --- | --- |
| 输入是 SQL NULL | 该行短路成 NULL（默认），或由 `Option<T>` 入参自定义语义 | 由参数类型决定，见 `../src/AGENTS.md` |
| 上游用哨兵表示「算不出」（NAN、`Option::None`、`Err`） | **NULL** | 一个共享 helper（statrs 是 `functions/mod.rs::nan_to_null`），所有包装都走它 |
| 参数存在但非法（尺度 ≤ 0、概率越界） | **查询报错** | `duck_error("sr_xxx: ...")`，消息里带函数名 |
| 结果是 ±∞ | **原样返回** | 无穷是合法结果，不要顺手折成 NULL |

铁律：哨兵值永远不能作为值出现在结果列里。空组算出 NAN 却把 NAN 返回给用户，是这个项目最容易犯
也最难被发现的错 —— 用户只会看到 `nan`，然后以为数据有问题。

### 有副作用 / 非确定性的标量函数：一律 `volatile = true`

两类函数必须标 `#[duck_scalar_function(volatile = true)]`：

- **结果不稳定**：采样、随机数、当前时间戳；
- **会碰外面世界**：打印到 stdout、写文件、起进程、发请求。

不标会发生什么（[duckfn_open](https://github.com/shijianjs/duckfn-open) 实测）：

- **常量折叠发生在优化期**。参数全是常量时，DuckDB 会在执行之前先求值一次、把调用折成常量；折出来的
  值如果又被外层表达式吃掉，执行期会**再求值一次**。于是 `SELECT length(op_write_file('a','txt'))`
  落**两个**文件，`SELECT length(op_print(...))` 打**两遍**。
- **`EXPLAIN` 会真的执行一次**。`EXPLAIN SELECT op_write_file('a','txt')` 会写出一个文件 —— 只做规划、
  不打算执行的查询不该有副作用。

标上之后语义变成「一次调用 = 一行，不多不少」，常量参数也一样：`SELECT op_print('x') FROM t` 有几行
就打几行，`SELECT op_open_path('report.html') FROM reports` 一行开一个窗口。这才是逐行标量函数该有的
样子，但要接受两点：

- **不要在任何文档里承诺「常量参数只跑一次」**。README、用户指南、`description.yml` 都按「逐行」写。
- **纯函数不要加**。加了就放弃了优化器的复用与折叠，没有收益。判断标准：同样输入调用两次，结果可能
  不同吗？会碰外面世界吗？任一为「是」→ 加。

配套的设计与实现约定：

- **返回值带上入参**：`op_print(text)` 返回那段文本、`op_write_file(content, ext)` 返回路径。副作用之外
  还有个值可用，用户可以把它留在投影里，也可以 `length(...)` 折叠掉只保留副作用。
- **写 stdout 要 flush**：Rust 的 stdout 按行缓冲，不带换行的输出会卡在缓冲区里，看起来像没打。
- **别承诺输出顺序**：逐行副作用的先后属于执行细节，并行扫描下不保证与表顺序一致。需要顺序的场景
  应该在文档里说清「不要依赖顺序」，而不是假装它稳定。

怎么测这类函数（可数的证据，而不是「看起来打了」）见 [`../test/AGENTS.md`](../test/AGENTS.md)。

## 3. 目录与代码组织

- **目录与上游模块树同构**，一眼能对上：上游 `src/statistics/` → 本仓库 `functions/statistics/`。
- `functions/mod.rs` 头注释放**对应表**（上游模块 → 本仓库文件）与「不包装清单 + 理由」，中英双语
  （项目注释风格见 statrs 的先例）。
- 一个函数组一个文件，文件名写清「哪一类 + 做什么」；长大后再拆子目录。
- 跨函数复用的东西（NULL 折算、概率校验、类型转换）放 `functions/mod.rs` 或对应子模块的
  `mod.rs`，不要每个文件抄一份。
- `types/` 只在需要自定义类型时启用（STRUCT / ENUM / `list<struct>` 行类型 / `DuckLazy` 配置），
  不需要就整个目录删掉。

具体写法（返回形状、`auto_collect`、注释与文档元数据）见 [`../src/AGENTS.md`](../src/AGENTS.md)。

## 4. 实现顺序：每批一个闭环

1. **先打通链路**：只注册 1 个聚合 + 1 个标量，跑 `just build` → `just test`。确认产物名、
   `LOAD`、`require`、CI 都对，再开始铺量。这一步失败时改动面最小，最容易定位。
2. **一个上游模块一批**：一批 = 一个文件 + 一份 `.test`。批内先写函数再写测试，测试不过就地修，
   不要把测试留到最后统一补。
3. **全量核对**：`SELECT count(*) FROM duckdb_functions() WHERE starts_with(function_name, 'sr_')`
   与能力清单对差集；差集要么补上，要么写进「不包装清单」。
4. **补文档**：`just docs_csv` 导出描述，`--all` 版本看哪些函数还没写 `description`。

批次之间不要顺手重构上一批的形状 —— 形状一旦定了，改一次要连带改所有 `.test` 与文档。

## 5. 测试策略

写法与覆盖矩阵见 [`../test/AGENTS.md`](../test/AGENTS.md)，这里只讲策略：

- **期望值只信上游的 `assert` 与实跑输出**，不手算。手算过一次协方差，错了，还写进了测试。
- **每个函数的断言数 ≥ 上游对它的用例数**，且包含上游的全部用例。上游覆盖不到、SQL 才有的情况
  （空组、`GROUP BY`、NULL 跳过/配对、跨 DataChunk、并行 `combine`、错误文案）在本仓库新建。
- **冒烟文件里的函数计数是活的**：加函数就要同步，否则它会在某次无关改动后突然变红，或者更糟 ——
  一直是错的却没人看。

## 6. 文档

三层，受众不同，不要混：

| 层 | 位置 | 写什么 | 不要写什么 |
| --- | --- | --- | --- |
| README（中英各一份） | `README.md` / `README.zh.md` | 这个扩展给 DuckDB 加了什么、怎么装、函数总表入口、两三个示例 | 目录结构、构建步骤 |
| 文档站 · 用户指南 | `docs/docs/user-guide/` | 安装与加载、函数参考（按上游模块分页） | Rust、`make`、`cargo` |
| 文档站 · 开发指南 | `docs/docs/development/` | 目录结构、写函数、测试、构建与发版 | 面向用户的函数清单 |

几条硬约定：

- **安装写法用三步**（`curl` 下载 → 改名 → `LOAD` 本地文件）。`LOAD` 不接受 URL，也不认
  `<扩展名>-<平台>.duckdb_extension` 这种带平台后缀的文件名（DuckDB 按文件名找入口点符号）。
  完整写法见根 [`AGENTS.md`](../AGENTS.md) 的「给用户的『安装发布产物』写法」。
- **函数多时别手写总表**：从 `duckdb_functions()` 导出再按上游模块分页，手写一定会漏。
- **`description` / `comment` / `example` 是社区扩展页的唯一来源**，DuckDB 的 C API 没有设置描述的
  接口。写英文、一句一条完整 SQL。`just docs_csv` 导出，`--all` 看缺口。
- **版本号写占位符 `{{EXTENSION_VERSION}}`**，别手写（发版脚本只更新 `docs/extension-version.ts`）。
- **中英成对改**：同一页在 `docs/docs/` 与 `docs/i18n/zh-Hans/.../current/` 各一份，相对路径、
  `id` / `slug` / `sidebar_position` 保持一致，页内链接用相对文件路径。细节见
  [`../docs/AGENTS.md`](../docs/AGENTS.md)。
- **图优先让扩展自己画**：返回值是 SVG / HTML 的扩展，文档里用可运行块的 `"show":"svg"` 渲染，不要贴
  截图 —— 截图会过期、锁死一个主题的配色、和块里的数据对不上。静态图（logo、社交卡片）放
  `docs/static/img/`，引用走 `useBaseUrl`；不要钉画布尺寸与预览框高度，交互内容必须走 `iframe` 并手调
  高度。完整的取舍见 [`../docs/AGENTS.md`](../docs/AGENTS.md) 的「展示图像」。

## 7. 发布与社区扩展

- 发版流程（`release_check` / `release_bump` / `release_tag` / `release_ci` / `release_dev`）见根
  [`AGENTS.md`](../AGENTS.md)，包装类项目没有额外步骤。
- 社区扩展：`community-extension/description.yml` 的 `version` 跟发版走，但 `repo.ref` 那个提交
  SHA 要人工更新到 tag 指向的提交；`docs/function_descriptions.csv` 从 `just docs_csv` 复制过去。
  字段依据见 [`../community-extension/AGENTS.md`](../community-extension/AGENTS.md)。
- 函数数量大的项目，社区页表格会很长：CSV 少一条就是页面上的一行空描述，**发版前跑一遍缺口检查**。

## 8. 验收清单

全量包装的项目，收工前逐条过：

- [ ] `functions/mod.rs` 头注释里有「上游模块 ↔ 本仓库文件」对应表，以及「不包装清单 + 理由」
- [ ] 上游每个 `pub fn` / `pub const` 都有 SQL 出口，或出现在不包装清单里
- [ ] `duckdb_functions()` 的计数与清单一致（注意 overload 会让计数大于函数名数）
- [ ] 三态契约写在一处，所有函数都走同一个 helper；结果列里不会出现 `nan`
- [ ] 有副作用 / 非确定性的函数都标了 `volatile = true`，测试里有 `has_side_effects` 名单、
      求值次数与「`EXPLAIN` 不产生副作用」三条断言
- [ ] 每个函数至少覆盖：正常值、NULL、边界、错误路径（能报错的才有）
- [ ] 聚合额外覆盖：空组、单值、`GROUP BY`、跨 DataChunk、`PRAGMA threads=4`
- [ ] 期望值全部来自上游 `assert` 或实跑输出，没有手算
- [ ] 冒烟文件里的函数计数已同步
- [ ] `just lint`（clippy `-D warnings`）干净
- [ ] `just test` 全绿，`just build_wasm` 能过（若保留 wasm 路线）
- [ ] 每个函数都有英文 `description`（`cargo run --bin duckfn -- function_descriptions --all` 无缺口）
- [ ] 文档里的图是块渲染出来的（不是截图），静态图走 `static/img/` + `useBaseUrl`，没有钉死画布或预览框尺寸
- [ ] README 中英同步、docs 站中英同步、安装写法是三步而不是 `LOAD '<url>'`
- [ ] `community-extension/description.yml` 的 `repo.ref` 指向已发布的 tag

## 9. 踩过的坑（duckfn_statrs / duckfn_open 实录）

1. **`list()` 保留 NULL，聚合跳过 NULL。** 把多列输入做成「`Vec` 参数 + 标量」时，用户得先
   `list(x)`，而 `list()` 会把 NULL 也收进去 —— 与聚合的语义不一致，同一个函数两种行为。
   这是「多列输入优先做成聚合」的第二条理由。
2. **NAN 泄漏。** 上游对空输入/样本不足返回 NAN，直接透出就是结果列里的 `nan`。必须统一折 NULL。
3. **手算期望值。** 协方差那组曾经手算，错了，还写进测试。只信上游 `assert` 与实跑输出。
4. **`range(10)` 测不出 `combine` 的 bug。** 一个 DataChunk 默认 2048 行，少于这个数永远不会走
   合并路径。跨 chunk 与 `PRAGMA threads=4` 的用例必须真的造够数据。
5. **`LOAD 'https://…'` 不会下载。** 下载是 `INSTALL '<url>'`；`LOAD` 只接受扩展名或本地路径。
6. **产物改名就加载失败。** DuckDB 按文件名找 `<扩展名>_init_c_api` 符号，`win.duckdb_extension`
   这种名字会报 `did not contain function ...`，不是产物坏了。
7. **上游 MSRV 抬高下限。** statrs 0.19 要 Rust 1.89；`rust-version` 不跟着改，CI 会在一个奇怪的
   地方失败。
8. **`checked_*` 与多类型重载让函数数虚高。** 只包一份、必要时用 `overloads_name`，别一个上游
   函数注册成五个。
9. **`docs_build` 依赖 wasm 产物。** 文档站预加载的是 `static/duckdb-extensions/` 下的本机构建，
   没先 `just build_wasm_eh` 会构建失败。
10. **副作用函数没标 `volatile`：`EXPLAIN` 也会落文件。** 常量折叠发生在优化期，折进 `length(...)`
    之后执行期还会再求值一次 —— 一个调用落两个文件、输出打两遍；`EXPLAIN SELECT op_write_file(...)`
    会真的写一个文件。标上 `volatile = true` 后语义变成「一次调用 = 一行」，代价是常量参数也逐行执行。
11. **stdout 不 flush 就看不到。** Rust 的 stdout 按行缓冲，`op_print(text, '')` 这种不带换行的调用
    不显式 flush，内容会卡在缓冲区里，看起来像没打或打了一半。
12. **注释与代码一起过期。** 目录结构、测试文件清单这类「第二份真相」最容易腐烂 —— 能在代码里读到的
    就不要抄进文档，抄了就写清「以代码为准」。
