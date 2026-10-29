---
title: 安装
sidebar_position: 2
description: 装 my_extension 的三条路 —— DuckDB 社区仓、GitHub Release，或本地自己构建的产物 —— 以及怎么确认它加载上了。
---

# 安装

`my_extension` 需要 **DuckDB 1.4 或更新**。发布产物基于 DuckDB 2.0.0 构建，同样能加载进 1.4.x / 1.5.x
（C API 二进制兼容），所以没有单独的 1.5 构建：旧版本用户直接安装同一份 2.0 release 资产即可。它是一个普通的可加载扩展，`INSTALL` 与 `LOAD` 就是全部；
没有额外的下载、库文件或配置步骤。

## 从社区仓安装

最省事的一条，也是唯一不需要任何命令行开关的：

```sql
INSTALL my_extension FROM community;
LOAD my_extension;
```

DuckDB 会下载一份签过名、且与你本机 DuckDB 版本和平台匹配的构建。`INSTALL` 只需做一次，但每个新会话
都要重新 `LOAD`。

:::note[关于这条路]

`INSTALL … FROM community` 要等扩展注册进 DuckDB 社区仓之后才可用。注册是维护者做的一步，见
[社区扩展](../development/community-extension.md)；在那之前，用 Release 文件或本地构建的产物。

:::

## 从 GitHub Release 安装

每个版本都会发成一个 GitHub Release，里面按平台各挂一份 `.duckdb_extension`，命名是
`my_extension-<平台>.duckdb_extension`（`windows_amd64`、`linux_amd64`、`osx_arm64` 等），另有一份给浏览器用的
WebAssembly 构建。下载你平台的那一份，**并改名成 `my_extension.duckdb_extension` 再加载** —— `LOAD` 收的是扩展名
或本地路径（它不会去下载 URL，下载那条路是 `INSTALL '<url>'`），而入口点符号由文件名决定，直接用 Release 里的
名字会报 `did not contain function "my_extension-windows_amd64_init_c_api"`：

```shell
curl -LO https://github.com/<owner>/<repo>/releases/latest/download/my_extension-windows_amd64.duckdb_extension
mv my_extension-windows_amd64.duckdb_extension my_extension.duckdb_extension
duckdb -unsigned -c "LOAD './my_extension.duckdb_extension';"
```

Release 里的文件没有用 DuckDB 的发行密钥签名，所以要加 `-unsigned`。

## 从本地构建的产物安装

如果你自己从源码构建了扩展，把 `LOAD` 指到产物上 —— 同样要用 `-unsigned` 启动 DuckDB：

```sql
LOAD './build/debug/my_extension.duckdb_extension';
```

怎么构建见[开发指南](../development/quick-start.md)。

## 确认它加载上了

`duckdb_extensions()` 列出可用的扩展，`duckdb_functions()` 列出可调用的函数：

```sql
SELECT extension_name, installed, loaded
FROM duckdb_extensions()
WHERE extension_name = 'my_extension';
```

```sql
SELECT function_name, function_type, return_type
FROM duckdb_functions()
WHERE function_name LIKE 'my%';
```

三个函数就是三行；如果一行都没有，说明你查的这个会话里没有加载这个扩展。
