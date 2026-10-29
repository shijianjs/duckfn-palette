---
title: Installation
sidebar_position: 2
description: The three ways to install duckfn_palette — DuckDB's community repository, a GitHub release, or a local build — and how to check that it loaded.
---

# Installation

`duckfn_palette` needs **DuckDB 1.4 or newer**. The release binaries are built against DuckDB 2.0.0 and
also load into 1.4.x and 1.5.x (the C API is binary compatible across these versions), so there is
no separate 1.5 build: on an older DuckDB, install the same 2.0 release asset. It is an ordinary loadable extension, so `INSTALL` and
`LOAD` are all it takes; there is no separate download, library or setup step.

## From the community repository

The simplest route, and the only one that needs no flags:

```sql
INSTALL duckfn_palette FROM community;
LOAD duckfn_palette;
```

DuckDB downloads a build that is signed and matched to your DuckDB version and platform. `INSTALL` is a
one-time step, but `LOAD` is needed again in each new session.

:::note[About this route]

`INSTALL … FROM community` works once the extension has been registered in DuckDB's community repository.
That registration is a maintainer step described in
[Community extensions](../development/community-extension.md); until it is done, load a release file or a
local build instead.

:::

## From a GitHub release

Every version is published as a GitHub Release carrying one `.duckdb_extension` per platform, named
`duckfn_palette-<platform>.duckdb_extension` (`windows_amd64`, `linux_amd64`, `osx_arm64`, and so on), plus a
WebAssembly build for the browser. Download the one matching your platform and **rename it to
`duckfn_palette.duckdb_extension`** before loading it — `LOAD` takes an extension name or a local path (it fetches
no URLs; that is what `INSTALL '<url>'` is for), and DuckDB takes the entry-point symbol from the file name, so
the release name fails with `did not contain function "duckfn_palette-windows_amd64_init_c_api"`:

```shell
curl -LO https://github.com/<owner>/<repo>/releases/latest/download/duckfn_palette-windows_amd64.duckdb_extension
mv duckfn_palette-windows_amd64.duckdb_extension duckfn_palette.duckdb_extension
duckdb -unsigned -c "LOAD './duckfn_palette.duckdb_extension';"
```

A release file is not signed by DuckDB's distribution key, hence `-unsigned`.

## From a local build

If you built the extension from source yourself, point `LOAD` at the artifact — and, again, start DuckDB
with `-unsigned`:

```sql
LOAD './build/debug/duckfn_palette.duckdb_extension';
```

Building it is the subject of the [Development guide](../development/quick-start.md).

## Checking that it loaded

`duckdb_extensions()` lists what is available, and `duckdb_functions()` what is callable:

```sql
SELECT extension_name, installed, loaded
FROM duckdb_extensions()
WHERE extension_name = 'duckfn_palette';
```

```sql
SELECT function_name, function_type, return_type
FROM duckdb_functions()
WHERE function_name LIKE 'my%';
```

Three rows for three functions; if the list is empty, the extension is not loaded in the session you are
querying.
