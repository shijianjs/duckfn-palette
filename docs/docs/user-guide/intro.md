---
title: Introduction
sidebar_position: 1
slug: /intro
description: What duckfn_palette adds to DuckDB, and how to start using it.
---

# Introduction

`duckfn_palette` is a [DuckDB loadable extension](https://duckdb.org/docs/stable/extensions/overview): it
adds SQL functions to DuckDB, and once it is loaded they behave like DuckDB's own. You do not need to
know how it was written or to compile anything — the CLI, a Python or R session and the browser build all
take the same `.duckdb_extension` file.

It brings palette's color types and algorithms into SQL: 145 functions under the `plt_` prefix covering
44 color spaces, conversion, blending and compositing, color difference and contrast, color schemes,
CSS and named colors, white points and chromatic adaptation, matrices, CAM16 and 8-bit packing. Two
supporting libraries fill the edges: csscolorparser for CSS color strings, and colorous for preset
gradients and categorical palettes.

A color is one `STRUCT`, so it can live in a column, be passed around as a value, and be compared:

```text
STRUCT(space VARCHAR, c1 DOUBLE, c2 DOUBLE, c3 DOUBLE, alpha DOUBLE)
```

## Install it

```sql
INSTALL duckfn_palette FROM community;
LOAD duckfn_palette;
```

Then call the functions in any query:

```sql {"type":"duckfn","show":"table"}
SELECT plt_to_hex(plt_convert(plt_srgb(1, 0.5, 0), 'oklch')) AS as_oklch,
       plt_difference(plt_named('red'), plt_named('blue'), 'ciede2000') AS delta_e;
```

That block runs right here, in your browser: the site preloads the extension from the project's latest
release, so there is no `LOAD` to write. [Installation](./installation.md) covers the other ways to get
the file.

## Where to go next

- [Installation](./installation.md) — the community repository, a release file, or a local build.
- [Functions](./functions.md) — every function, grouped by what it does, with examples you can run.
- [Development guide](../development/quick-start.md) — building the extension from source. That part is
  for people working on this repository; it is not needed to use the extension.
