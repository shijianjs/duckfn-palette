# AGENTS.md —— test/ 的测试约定

这份文件是给改 `test/sql/` 的人（含 AI agent）看的。跑法（`just test`、直连 `duckdb_sqllogictest`、
`make` 在 Windows 上的限制）在 [`../docs/docs/development/testing.md`](../docs/docs/development/testing.md)，
本文件只写**写什么、期望值从哪来、哪些边界不能漏**。

本仓库的测试只有 SQLLogicTest 一种形式，没有 Rust 单测：包装类项目的函数体几乎都是「一行转发给
上游」，要测的是 SQL 侧的行为（NULL、分组、并行、错误），而不是那行转发本身。

## 文件怎么分

```text
test/sql/<扩展名>.test        冒烟：LOAD 前后、注册数量盘点
test/sql/<上游模块>.test      一个上游模块一份（与 src/ 的目录同构）
test/sql/<上游模块>/<子模块>.test   模块大了再分子目录
```

- **冒烟文件只有一份**，内容是「加载前函数不存在 → `require` 后存在 → `duckdb_functions()` 计数」。
- **功能文件按上游模块分**，不按「标量/聚合」分 —— 与 `src/` 的划分保持一致，改一个模块时能一眼
  找到它的测试。
- 每份 `.test` 开头写 `# name:` / `# description:` / `# group:`，`description` 里写清**期望值的来源**
  （哪个上游版本、哪几个用例、哪些是本仓库新增的）。这是后来者判断「能不能改这个数」的唯一线索。

## 期望值：只信上游 `assert` 与实跑输出

**不要手算。** 手算过一次协方差，错了，还写进了测试 —— 之后所有依赖它的判断都错。

期望值的合法来源，按优先级：

1. **上游 doc example（doctest）里的 `assert_eq!`**：把输入照抄成 `VALUES`，期望值照抄。
2. **上游 `#[cfg(test)] mod tests` 里的断言**：同上。
3. **实跑输出**：先在 `just repl` 里跑一遍，把输出原样贴进 `.test`。浮点用科学计数法原样写，
   不要为了好看截断。

上游覆盖不到、只有 SQL 才有的情况（空组、`GROUP BY`、NULL 跳过/配对、跨 DataChunk、并行
`combine`、错误文案），在本仓库新建用例，期望值同样来自实跑。

**每个函数的断言数 ≥ 上游对它的用例数**，且包含上游的全部用例。上游有 3 个 assert，这里就不能只有
1 个。

## 每个函数的最小覆盖

| 情况 | 例子 |
| --- | --- |
| 正常值 | 上游 doc example 那组输入 |
| NULL 输入 | `(VALUES (1.0), (NULL), (3.0))`，确认跳过而不是整列变 NULL |
| 边界值 | 分布参数退化、概率 0/1、空组、单值 |
| 错误路径 | 参数非法（尺度 ≤ 0、概率越界）→ `statement error` |
| 参数来自列 | 不是常量字面量时同样正确（防止只在常量折叠路径下对） |
| 跨 DataChunk | 超过 2048 行（一个 DataChunk 默认行数） |
| 返回类型 | `typeof(...)`，确认 `LIST(DOUBLE)` / `UBIGINT` 之类没被悄悄改掉 |

聚合额外覆盖：**空组**（`WHERE 1 = 0` → NULL，不是 0）、**单值**（样本方差无定义 → NULL）、
**`GROUP BY`**（整组 NULL 的那一组回 NULL）、**并行 `combine`**（`PRAGMA threads=4` 下结果与单线程
一致）。

非确定性函数（采样）不能断言具体值，断言形状与范围：`list_count`、`list_min >= 0`、元素个数等于
`k`、两次调用不保证相等。

## 副作用函数怎么测

打印、写文件、起进程这类函数，**副作用本身不进结果集**，所以不能断言「打了什么」，要断言**可数的
证据**。四种，缺一不可：

1. **目录断言**：注册成 `volatile` 之后，`duckdb_functions()` 里的 `has_side_effects` 就是 true。
   把该为 true 的名字集合整体断言一次，防止后来新增函数忘了标：

   ```sql
   query T
   SELECT string_agg(DISTINCT function_name, ',' ORDER BY function_name)
   FROM duckdb_functions()
   WHERE function_name LIKE 'my_%' AND has_side_effects;
   ----
   my_print,my_write_file
   ```

2. **求值次数**：一次调用只产生一个副作用。用文件数当计数器 —— 没有 `volatile` 时，
   `SELECT length(my_write_file(...))` 会写两个文件。

   ```sql
   query T
   SELECT length(my_write_file('body', 'txt', {'dir':'__TEST_DIR__/once'})) > 0;
   ----
   true
   query I
   SELECT count(*) FROM glob('__TEST_DIR__/once/*');
   ----
   1
   ```

3. **`EXPLAIN` 不产生副作用**：`statement ok` 跑一条 `EXPLAIN`，然后确认目录里是 0 个文件。这是
   「常量折叠发生在优化期」那条最直接的回归断言。

4. **常量参数逐行执行**：`FROM (VALUES (1),(2),(3))` 三行就是三个文件。这条同时锁住了「一次调用 =
   一行」的语义，防止有人为了「常量只跑一次」把 `volatile` 去掉。

补充约定：

- **写出去的东西要读回来验**：`read_text` / `read_blob` 比对内容，别只断言文件存在。
- **测试目录用 `__TEST_DIR__`**，不要写死路径；`just test` 收尾会清掉被忽略的临时文件。
- **不要断言 stdout 的内容或顺序**：stdout 是进程级的，sqllogictest 不比较它，并行下顺序也不保证。
  要证明「确实执行了」，用文件数或返回值。

## 写法约定

- 每份文件从**干净数据库**开始，所以第一份可以断言「加载前函数不存在」，其余文件开头 `require
  <扩展名>`。
- 数值断言优先**定型**再比：`my_stat(x)::DECIMAL(12,6)`，这样返回类型变了也不会连带改期望值。
  需要近似比较时用 `query R` 原样写浮点。
- `query` 后面的类型标记：`I` 整数、`R` 浮点（近似比较）、`T` 文本，一个字符对应一列。
- `statement error` 下面是**子串匹配**，写有辨识度的一段即可（例如 `must be within [0, 1]`），
  不要抄整条 DuckDB 错误消息 —— 那种断言会随版本更新碎掉。
- 用例之间不要共享状态：每个 `query` 自己造数据（`VALUES` / `range`），不依赖上一条的结果。
- 临时文件写到 `__TEST_DIR__` 下（runner 会注入），不要写死路径。

## 冒烟文件的计数是活的

```sql
query I
SELECT count(*) FROM duckdb_functions() WHERE starts_with(function_name, 'my_');
----
3
```

- 加/删函数就同步这三个数（总数、聚合数、标量数）。**先跑一遍数出来再填，不要凭空改。**
- 注意 `duckdb_functions()` 按**签名**计数：一个函数有多个 overload 会占多行，所以数字可能大于
  函数名个数。改动 overload 时尤其要重新数。

## 不要为了让测试变绿而改期望值

测试红了先判断是哪一类：

- **代码错了** → 改代码。这是绝大多数情况。
- **上游行为变了**（升级上游 crate 后） → 改期望值，并在文件头的 `description` 里记下「上游 x.y
  起行为变化」，同时检查文档是否也要改。
- **期望值当初就是手算的** → 按上游 `assert` 或实跑输出重写，不要在原值上微调。

为了让测试通过而把期望值改成当前输出，等于把 bug 固化成契约 —— 这是这个项目最贵的错误。

## 新增一批函数时的顺序

1. 先写正常值（上游 doc example），确认函数注册、类型正确；
2. 再补 NULL / 空组 / 边界 / 错误路径；
3. 最后补跨 DataChunk 与 `PRAGMA threads=4`；
4. 同步冒烟计数；
5. `just test` 全绿后再提交（`make test` 不会自动重建，改完 Rust 先 `just ci-build`）。
