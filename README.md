[English](README.md) | [简体中文](README.zh.md)

# duckfn_palette

[![DuckDB 1.4+](https://img.shields.io/badge/DuckDB-1.4%2B-fff100?logo=duckdb&style=flat)](https://duckdb.org/)
[![Weekly Downloads](https://img.shields.io/badge/dynamic/json?url=https%3A%2F%2Fcommunity-extensions.duckdb.org%2Fdownloads-last-week.json&query=%24.duckfn_palette&label=Weekly%20Downloads&color=blue&logo=duckdb&logoColor=a899ff&style=flat)](https://duckdb.org/community_extensions/extensions/duckfn_palette)
[![Release](https://img.shields.io/github/v/release/shijianjs/duckfn-palette?style=flat&logo=github)](https://github.com/shijianjs/duckfn-palette/releases)
[![License](https://img.shields.io/badge/license-MIT-14459b.svg?style=flat&logo=github)](https://github.com/shijianjs/duckfn-palette/blob/main/LICENSE)

`duckfn_palette` wraps three Rust color crates with SQL functions: [palette](https://crates.io/crates/palette)
for 44 color spaces, conversion, blending, color difference and contrast, schemes, white points, CAM16
and 8-bit packing; [csscolorparser](https://crates.io/crates/csscolorparser) for CSS Color Level 4
strings in both directions; and [colorous](https://crates.io/crates/colorous) for the
d3-scale-chromatic gradients and categorical palettes. Together they register 160 scalar functions
under the `plt_` prefix.

A color is a single STRUCT in SQL:

```sql
plt_color(space VARCHAR, c1 DOUBLE, c2 DOUBLE, c3 DOUBLE, alpha DOUBLE)
```

Every space has a constructor of the same name (`plt_srgb`, `plt_oklch`, `plt_cam16_jch`, …) taking
arguments in palette's `Color::new` order; `plt_convert` moves between any two spaces and keeps the
space tag and alpha along the way. Written with [duckfn](https://crates.io/crates/duckfn): attribute
macros register plain Rust functions as DuckDB scalar functions, with no C++ build involved.

## Quick start

```shell
make configure   # once: builds configure/venv (Python + the sqllogictest runner)
make debug       # -> build/debug/duckfn_palette.duckdb_extension
```

Locally built extensions are unsigned, so DuckDB has to be started with `-unsigned`:

```shell
duckdb -unsigned -c "
LOAD './build/debug/duckfn_palette.duckdb_extension';
SELECT plt_to_hex(plt_convert(plt_srgb(1, 0.5, 0), 'oklch'));             -- #ff8000
SELECT plt_to_hex(plt_mix(plt_named('red'), plt_named('blue'), 0.5));     -- #800080
SELECT plt_contrast(plt_named('white'), plt_named('black'));              -- 21.0
"
```

The `Justfile` wraps the same commands: `just build`, `just sql "SELECT plt_to_hex(plt_named('red'))"`,
`just repl` (a REPL with the extension already loaded).

## Functions

160 distinct names, 169 signatures (overloads share a name). The full signatures and runnable examples
are in the [function reference](docs/docs/user-guide/functions/overview.md).

| Category | Representative functions |
| --- | --- |
| Color structure | `plt_color`, `plt_space`, `plt_components`, `plt_component`, `plt_alpha`, `plt_with_alpha`, `plt_opaque`, `plt_from_components`, `plt_color_spaces` |
| Constructors (one per space) | `plt_srgb`, `plt_oklch`, `plt_lab`, `plt_xyz`, `plt_hsl`, `plt_luma`, … 44 spaces |
| Conversion | `plt_convert`, `plt_convert_clamped`, `plt_clamp`, `plt_is_within_bounds`, `plt_equals`, `plt_luma`, `plt_luma_linear`, `plt_luma_gamma` |
| Manipulation | `plt_mix`, `plt_lighten`, `plt_darken`, `plt_brighten`, `plt_greyscale`, `plt_tint`, `plt_shade`, `plt_saturate`, `plt_desaturate` (each with `_fixed`), `plt_shift_hue`, `plt_with_hue`, `plt_hue`, `plt_premultiply`, `plt_unpremultiply` |
| Blending & compositing | `plt_blend`, `plt_compose`, `plt_blend_equations`, `plt_blend_modes`, `plt_compose_modes`, `plt_blend_parameters` |
| Difference & contrast | `plt_difference`, `plt_difference_metrics`, `plt_relative_luminance`, `plt_contrast`, `plt_contrast_check`, `plt_contrast_criteria`, `plt_most_readable`, `plt_brightness`, `plt_is_light`, `plt_is_dark` |
| Color schemes | `plt_complementary`, `plt_split_complementary`, `plt_analogous`, `plt_analogous_secondary`, `plt_triadic`, `plt_tetradic`, `plt_monochromatic`, `plt_harmony`, `plt_color_schemes` |
| Named colors / CSS | `plt_named`, `plt_named_names`, `plt_nearest_named`, `plt_css_parse`, `plt_css_is_valid`, `plt_css_name`, `plt_css_serialize`, `plt_css_formats`, `plt_from_hex`, `plt_to_hex`, `plt_to_hex6`, `plt_to_hex8`, `plt_to_percentage_rgb`, `plt_to_hsv_string` |
| White points, adaptation, encoding | `plt_white_points`, `plt_white_point_xyz`, `plt_white_point_xy`, `plt_adaptation_methods`, `plt_adaptation_matrix`, `plt_lms_matrix`, `plt_lms_to_xyz_matrix`, `plt_diagonal_adaptation_matrix`, `plt_chromatic_adapt`, `plt_encoding_spaces`, `plt_transfer_function`, `plt_transfer_functions`, `plt_decode`, `plt_encode`, `plt_white_point_of`, `plt_rgb_primaries`, `plt_rgb_to_xyz_matrix`, `plt_xyz_to_rgb_matrix` |
| Matrices | `plt_matrix_identity`, `plt_matrix_scale`, `plt_matrix_multiply`, `plt_matrix_inverse`, `plt_matrix_apply`, `plt_apply_matrix` |
| 8-bit packing | `plt_pack`, `plt_unpack`, `plt_pack_orders` |
| CAM16 | `plt_cam16_params`, `plt_cam16_params_xyz`, `plt_cam16_default_params`, `plt_cam16_surrounds`, `plt_cam16_spaces`, `plt_cam16_from_xyz`, `plt_cam16_to_xyz`, `plt_cam16_partial_from_xyz`, `plt_cam16_partial_to_xyz` |
| colorous presets | `plt_colorous_palettes`, `plt_colorous_gradient`, `plt_colorous_palette`, `plt_colorous_categoricals`, `plt_colorous_categorical` |
| Random colors | `plt_random_spaces`, `plt_random_color`, `plt_random_colors` |

Three rules hold across every function:

- **NULL in, NULL out.** A NULL argument yields NULL; so does anything palette cannot compute (an
  undefined space combination, for example) — never `NaN` or 0.
- **A rejected argument fails the query**, and the message starts with the function name
  (`plt_convert: unknown color space 'nope'`).
- **Name arguments are case-insensitive** and treat hyphens and spaces as underscores: `'OKLCH'` and
  `'blue-green'` both work.

## Build from source

The official DuckDB `extension-ci-tools` makefiles are the build path:

```shell
make configure           # once: builds configure/venv (Python + the sqllogictest runner)
make debug               # -> build/debug/duckfn_palette.duckdb_extension
```

`make release` is the optimized version of the same flow. On Windows `make` has to run inside Git Bash.
The `Justfile` wraps it (`just build` = `make configure && make debug`, `just ci-build`, `just test`,
`just ci-release`).

## Testing

Tests are SQLLogicTest files under `test/sql/`:

```shell
just test          # make configure + make debug + make test
just ci-build      # just the official build, without running the tests
```

`make test` does not rebuild, so run `just ci-build` (or `make debug`) first after touching Rust code.
The development guide's [testing](docs/docs/development/testing.md) page has the iteration loop (running
the runner straight against `build/debug/*.duckdb_extension`) and what each test file covers.

## WebAssembly

```shell
just config_env   # once: pin the toolchain and add the wasm target
just build_wasm
```

The wasm build uses `src/wasm_lib.rs` (a `staticlib` mirror of `src/lib.rs`); the two crate roots must
always declare the same set of `mod`s.

## Documentation site

The repository carries a [Docusaurus](https://docusaurus.io/) site in `docs/`, in English and
Simplified Chinese, with a workflow that publishes it to GitHub Pages on every version tag:

```shell
just docs_install    # once
just docs_start      # dev server at http://localhost:3000
just docs_build      # the check that matters: onBrokenLinks is set to throw
```

The pages carry runnable SQL blocks (powered by
[`duckfn-docs-kit`](https://www.npmjs.com/package/duckfn-docs-kit)) that call the extension right in the
browser; `cd docs && npm test` re-runs them. The conventions — layout, commands, translation workflow,
deployment, the `{{EXTENSION_VERSION}}` version placeholder — are in [`docs/README.md`](docs/README.md).

## Installing the released extension

Releases are GitHub Releases carrying the build matrices' `.duckdb_extension` files, one per platform, named
`duckfn_palette-<platform>.duckdb_extension`. Download the one for your platform and **rename it to
`duckfn_palette.duckdb_extension`** before loading it — `LOAD` takes an extension name or a local path (it fetches
no URLs; that is what `INSTALL '<url>'` is for), and DuckDB takes the entry-point symbol from the file name, so
the release name fails with `did not contain function "duckfn_palette-windows_amd64_init_c_api"`:

```shell
curl -LO https://github.com/shijianjs/duckfn-palette/releases/latest/download/duckfn_palette-windows_amd64.duckdb_extension
mv duckfn_palette-windows_amd64.duckdb_extension duckfn_palette.duckdb_extension
duckdb -unsigned -c "LOAD './duckfn_palette.duckdb_extension';"
```

Publishing to DuckDB's [community extensions](https://duckdb.org/community_extensions/list_of_extensions)
makes it `INSTALL duckfn_palette FROM community` instead; the two files that requires are prepared in
[`community-extension/`](community-extension/AGENTS.md).

## Documentation

| File | What is in it |
| --- | --- |
| [AGENTS.md](AGENTS.md) | conventions, the duckfn knowledge map, the release flow |
| [docs/docs/development/](docs/docs/development/quick-start.md) | the development guide: directory layout, skeleton trade-offs, writing and testing functions, build & release (English and Simplified Chinese) |
| [docs/README.md](docs/README.md) | the documentation site: layout, commands, translations, deployment |
| [README.zh.md](README.zh.md) | this file, in Chinese |
