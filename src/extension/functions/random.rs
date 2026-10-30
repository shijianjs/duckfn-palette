// ============================================================================
// 随机色（palette 的随机采样 + 可复现的种子）。
//
// 参数非法（未知空间、没有随机分布的空间、负数个数）→ 查询错误；NULL 由参数类型短路。
// 种子写在参数里，所以这些函数是确定性的：同一组参数恒定得到同一批颜色，测试与
// `EXPLAIN` 都不会改变结果。
//
// Random colors from palette's distributions, seeded for reproducibility. Invalid spaces or
// negative counts are query errors; NULL short-circuits through the argument types.
// ============================================================================

use duckfn::{DuckOptionResult, duck_scalar_function};

use crate::extension::functions::err;
use crate::extension::types::color::PltColor;
use crate::extension::types::random;

/// 支持随机采样的空间。
#[duck_scalar_function(
    description = "Every color space that can be sampled randomly (palette implements a Standard distribution for it)",
    example = "SELECT plt_random_spaces()"
)]
fn plt_random_spaces() -> Vec<String> {
    random::RANDOM_SPACES
        .iter()
        .map(|s| s.to_string())
        .collect()
}

/// 采一个随机色。
#[duck_scalar_function(
    description = "A reproducible random color in the given space; the BIGINT seed identifies the sequence",
    example = "SELECT plt_random_color('oklch', 42)"
)]
fn plt_random_color(space: String, seed: i64) -> DuckOptionResult<PltColor> {
    Ok(Some(random::random_color(
        "plt_random_color",
        &space,
        seed as u64,
    )?))
}

/// 用同一序列采多个随机色。
#[duck_scalar_function(
    description = "A LIST of reproducible random colors from one seeded sequence; count must be non-negative",
    example = "SELECT plt_random_colors('srgb', 5, 42)"
)]
fn plt_random_colors(space: String, count: i64, seed: i64) -> DuckOptionResult<Vec<PltColor>> {
    let count = usize::try_from(count).map_err(|_| {
        err(format!(
            "plt_random_colors: count must be non-negative, got {count}"
        ))
    })?;
    Ok(Some(random::random_colors(
        "plt_random_colors",
        &space,
        count,
        seed as u64,
    )?))
}
