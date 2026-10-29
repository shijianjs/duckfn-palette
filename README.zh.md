[English](README.md) | [简体中文](README.zh.md)

# duckfn_palette

[![DuckDB 1.4+](https://img.shields.io/badge/DuckDB-1.4%2B-fff100?logo=duckdb&style=flat)](https://duckdb.org/)
[![周下载](https://img.shields.io/badge/dynamic/json?url=https%3A%2F%2Fcommunity-extensions.duckdb.org%2Fdownloads-last-week.json&query=%24.duckfn_palette&label=%E5%91%A8%E4%B8%8B%E8%BD%BD&color=blue&logo=duckdb&logoColor=a899ff&style=flat)](https://duckdb.org/community_extensions/extensions/duckfn_palette)
[![版本](https://img.shields.io/github/v/release/shijianjs/duckfn-palette?style=flat&logo=github&label=%E7%89%88%E6%9C%AC)](https://github.com/shijianjs/duckfn-palette/releases)
[![许可证](https://img.shields.io/badge/%E8%AE%B8%E5%8F%AF%E8%AF%81-MIT-14459b.svg?style=flat&logo=github)](https://github.com/shijianjs/duckfn-palette/blob/main/LICENSE)

`duckfn_palette` 把 Rust 的 [palette](https://crates.io/crates/palette) 颜色库搬上 SQL：145 个
`plt_` 前缀的标量函数，覆盖 44 个颜色空间、空间转换、混合与合成、色差与对比度、色彩理论、
CSS 与命名色、白点与色适应、矩阵运算、CAM16 和 8 位打包。另有两点补充：csscolorparser 负责
CSS 颜色字符串的正反序列化，colorous 提供预设渐变与分类色板。

颜色在 SQL 里就是一个 STRUCT：

```sql
plt_color(space VARCHAR, c1 DOUBLE, c2 DOUBLE, c3 DOUBLE, alpha DOUBLE)
```

每个空间都有同名构造器（`plt_srgb`、`plt_oklch`、`plt_cam16_jch` …），参数顺序与 palette 的
`Color::new` 一致；`plt_convert` 在任意两个空间之间转换，空间标签和 alpha 一路保留。用
[duckfn](https://crates.io/crates/duckfn) 编写：属性宏把普通 Rust 函数注册成 DuckDB 标量函数，
全程不碰 C++ 构建。

## 快速上手

```shell
make configure   # 只做一次：建 configure/venv（Python 与 sqllogictest 运行器）
make debug       # -> build/debug/duckfn_palette.duckdb_extension
```

自己构建的产物没有签名，加载时必须给 DuckDB 加 `-unsigned`：

```shell
duckdb -unsigned -c "
LOAD './build/debug/duckfn_palette.duckdb_extension';
SELECT plt_to_hex(plt_convert(plt_srgb(1, 0.5, 0), 'oklch'));                     -- #ff8000
SELECT plt_to_hex(plt_mix(plt_named('red'), plt_named('blue'), 0.5));             -- #800080
SELECT plt_contrast(plt_named('white'), plt_named('black'));                      -- 21.0
"
```

`Justfile` 把同样的命令包了一层：`just build`、`just sql "SELECT plt_to_hex(plt_named('red'))"`、
`just repl`（已 LOAD 扩展的 REPL）。

## 函数总览

一共 145 个名字（152 个签名，重载合并为同一个名字）。完整签名与可运行示例见
[函数参考](docs/i18n/zh-Hans/docusaurus-plugin-content-docs/current/user-guide/functions.md)。

| 类别 | 代表函数 |
| --- | --- |
| 颜色结构 | `plt_color`、`plt_space`、`plt_components`、`plt_component`、`plt_alpha`、`plt_with_alpha`、`plt_opaque`、`plt_from_components`、`plt_color_spaces` |
| 构造器（每空间一个） | `plt_srgb`、`plt_oklch`、`plt_lab`、`plt_xyz`、`plt_hsl`、`plt_luma` … 共 44 个空间 |
| 转换 | `plt_convert`、`plt_convert_clamped`、`plt_clamp`、`plt_is_within_bounds`、`plt_luma`、`plt_luma_linear`、`plt_luma_gamma` |
| 操作 | `plt_mix`、`plt_lighten`、`plt_darken`、`plt_saturate`、`plt_desaturate`（各带 `_fixed`）、`plt_shift_hue`、`plt_with_hue`、`plt_hue`、`plt_premultiply`、`plt_unpremultiply` |
| 混合与合成 | `plt_blend`、`plt_compose`、`plt_blend_equations`、`plt_blend_modes`、`plt_compose_modes`、`plt_blend_parameters` |
| 色差与对比度 | `plt_difference`、`plt_difference_metrics`、`plt_relative_luminance`、`plt_contrast`、`plt_contrast_check`、`plt_contrast_criteria` |
| 色彩理论 | `plt_complementary`、`plt_split_complementary`、`plt_analogous`、`plt_analogous_secondary`、`plt_triadic`、`plt_tetradic`、`plt_harmony`、`plt_color_schemes` |
| 命名色 / CSS | `plt_named`、`plt_named_names`、`plt_css_parse`、`plt_css_is_valid`、`plt_css_name`、`plt_css_serialize`、`plt_css_formats`、`plt_from_hex`、`plt_to_hex` |
| 白点、色适应、编码 | `plt_white_points`、`plt_white_point_xyz`、`plt_white_point_xy`、`plt_adaptation_methods`、`plt_adaptation_matrix`、`plt_lms_matrix`、`plt_lms_to_xyz_matrix`、`plt_diagonal_adaptation_matrix`、`plt_chromatic_adapt`、`plt_encoding_spaces`、`plt_transfer_function`、`plt_transfer_functions`、`plt_decode`、`plt_encode`、`plt_white_point_of`、`plt_rgb_primaries`、`plt_rgb_to_xyz_matrix`、`plt_xyz_to_rgb_matrix` |
| 矩阵 | `plt_matrix_identity`、`plt_matrix_scale`、`plt_matrix_multiply`、`plt_matrix_inverse`、`plt_matrix_apply`、`plt_apply_matrix` |
| 8 位打包 | `plt_pack`、`plt_unpack`、`plt_pack_orders` |
| CAM16 | `plt_cam16_params`、`plt_cam16_params_xyz`、`plt_cam16_default_params`、`plt_cam16_surrounds`、`plt_cam16_spaces`、`plt_cam16_from_xyz`、`plt_cam16_to_xyz`、`plt_cam16_partial_from_xyz`、`plt_cam16_partial_to_xyz` |
| colorous 预设 | `plt_colorous_palettes`、`plt_colorous_gradient`、`plt_colorous_palette`、`plt_colorous_categoricals`、`plt_colorous_categorical` |
| 随机色 | `plt_random_spaces`、`plt_random_color`、`plt_random_colors` |

三条贯穿全部函数的规则：

- **NULL 进，NULL 出。** 任一参数是 NULL 就返回 NULL；palette 算不出结果（例如未定义的空间组合）
  同样返回 NULL，而不是 `NaN` 或 0。
- **非法参数让查询失败**，错误信息以函数名开头（`plt_convert: unknown color space 'nope'`）。
- **名字参数大小写不敏感**，连字符与空格视作下划线：`'OKLCH'`、`'blue-green'` 都能用。

## 从源码构建

构建走官方 DuckDB `extension-ci-tools` makefile：

```shell
make configure           # 只做一次：建 configure/venv（Python 与 sqllogictest 运行器）
make debug               # -> build/debug/duckfn_palette.duckdb_extension
```

`make release` 是带优化的同一套流程。Windows 上 `make` 需要在 Git Bash 里跑。
`Justfile` 把它包了一层（`just build` = `make configure && make debug`、`just ci-build`、`just test`、
`just ci-release`）。

## 测试

测试用 SQLLogicTest 格式写在 `test/sql/` 下：

```shell
just test          # make configure + make debug + make test
just ci-build      # 只做官方构建，不跑测试
```

`make test` 不会自动重新构建，改完 Rust 先跑 `just ci-build`（或 `make debug`）。
更快的迭代方式（拿运行器直接跑 `build/debug/*.duckdb_extension`）与各测试文件覆盖什么，
见开发指南的[测试](docs/i18n/zh-Hans/docusaurus-plugin-content-docs/current/development/testing.md)。

## WebAssembly

```shell
just config_env   # 只做一次：固定工具链版本并装上 wasm target
just build_wasm
```

wasm 构建走 `src/wasm_lib.rs`（`src/lib.rs` 的 `staticlib` 镜像），两个 crate root 必须始终声明同一组
`mod`。

## 文档站

仓库里带着一个 [Docusaurus](https://docusaurus.io/) 站点（`docs/`），中英双语，并且有工作流在每次版本
tag 时把它发布到 GitHub Pages：

```shell
just docs_install    # 只做一次
just docs_start      # 本地预览 http://localhost:3000
just docs_build      # 真正该跑的那一条：onBrokenLinks 设为 throw，链接断了就构建失败
```

页面里带着可运行的 SQL 块（由
[`duckfn-docs-kit`](https://www.npmjs.com/package/duckfn-docs-kit) 提供），在浏览器里直接调用本扩展；
`cd docs && npm test` 会把它们重跑一遍。约定（目录、命令、翻译流程、部署、`{{EXTENSION_VERSION}}`
版本占位符）写在 [`docs/README.md`](docs/README.md) 里。

## 安装已发布的扩展

发布物是 GitHub Release 上各平台构建出的 `.duckdb_extension`，命名是 `duckfn_palette-<平台>.duckdb_extension`。
下载你平台那一份之后**先改名成 `duckfn_palette.duckdb_extension` 再加载** —— `LOAD` 收的是扩展名或本地路径
（它不会去下载 URL，下载那条路是 `INSTALL '<url>'`），而入口点符号由文件名决定，直接用 Release 里的名字会报
`did not contain function "duckfn_palette-windows_amd64_init_c_api"`：

```shell
curl -LO https://github.com/shijianjs/duckfn-palette/releases/latest/download/duckfn_palette-windows_amd64.duckdb_extension
mv duckfn_palette-windows_amd64.duckdb_extension duckfn_palette.duckdb_extension
duckdb -unsigned -c "LOAD './duckfn_palette.duckdb_extension';"
```

注册进 DuckDB 的[社区扩展](https://duckdb.org/community_extensions/list_of_extensions)之后，就变成
一句 `INSTALL duckfn_palette FROM community`；那需要提交的两份文件已经备在
[`community-extension/`](community-extension/AGENTS.md)。

## 文档

| 文件 | 里面有什么 |
| --- | --- |
| [AGENTS.md](AGENTS.md) | 约定、duckfn 知识地图、发版流程 |
| [docs/i18n/zh-Hans/…/development/](docs/i18n/zh-Hans/docusaurus-plugin-content-docs/current/development/quick-start.md) | 开发指南：目录结构、骨架取舍、写函数与测试、构建与发版 |
| [docs/README.md](docs/README.md) | 文档站：目录、命令、翻译流程、部署 |
| [README.md](README.md) | 本文件，英文 |
