// ============================================================================
// colorous 预设色板（连续渐变与分类色板）。
//
//   - `plt_colorous_gradient(name, t)`：在 [0, 1] 的 t 处取色（越界自动夹紧）；
//   - `plt_colorous_palette(name, n)`：均匀取 n 个颜色（n ≥ 1）；
//   - `plt_colorous_categorical(name)`：整组分类色板。
//
// 名字列表由 `plt_colorous_palettes()` / `plt_colorous_categoricals()` 给出；未知名字是
// 查询错误。颜色一律返回 sRGB（8 位色板除以 255）。
//
// colorous presets: sample a gradient at t, sample n evenly spaced colors, or take a whole
// categorical palette. Unknown names are query errors; colors come back as sRGB.
// ============================================================================

use duckfn::{DuckOptionResult, duck_scalar_function};

use crate::extension::functions::err;
use crate::extension::types::color::PltColor;
use crate::extension::types::colorous;

/// 全部连续渐变名。
#[duck_scalar_function(
    description = "Every colorous continuous gradient name (sequential, diverging and cyclical)",
    example = "SELECT plt_colorous_palettes()"
)]
fn plt_colorous_palettes() -> Vec<String> {
    colorous::GRADIENTS.iter().map(|s| s.to_string()).collect()
}

/// 全部分类色板名。
#[duck_scalar_function(
    description = "Every colorous categorical palette name",
    example = "SELECT plt_colorous_categoricals()"
)]
fn plt_colorous_categoricals() -> Vec<String> {
    colorous::CATEGORICALS.iter().map(|s| s.to_string()).collect()
}

/// 在渐变的 t 处取色。
#[duck_scalar_function(
    description = "Sample a colorous gradient at position t (clamped to [0, 1]), as an sRGB color",
    example = "SELECT plt_colorous_gradient('viridis', 0.5)"
)]
fn plt_colorous_gradient(name: String, t: f64) -> DuckOptionResult<PltColor> {
    let gradient = colorous::gradient("plt_colorous_gradient", &name)?;
    Ok(Some(colorous::to_color(gradient.eval_continuous(t))))
}

/// 均匀取 n 个渐变颜色。
#[duck_scalar_function(
    description = "n colors evenly sampled from a colorous gradient, as a LIST of sRGB colors; n must be at least 1",
    example = "SELECT plt_colorous_palette('plasma', 5)"
)]
fn plt_colorous_palette(name: String, n: i64) -> DuckOptionResult<Vec<PltColor>> {
    let gradient = colorous::gradient("plt_colorous_palette", &name)?;
    let n = usize::try_from(n).map_err(|_| {
        err(format!(
            "plt_colorous_palette: n must be at least 1, got {n}"
        ))
    })?;
    if n == 0 {
        return Err(err("plt_colorous_palette: n must be at least 1, got 0"));
    }
    let colors = (0..n)
        .map(|i| colorous::to_color(gradient.eval_rational(i, n)))
        .collect();
    Ok(Some(colors))
}

/// 整组分类色板。
#[duck_scalar_function(
    description = "A whole colorous categorical palette, as a LIST of sRGB colors",
    example = "SELECT plt_colorous_categorical('tableau10')"
)]
fn plt_colorous_categorical(name: String) -> DuckOptionResult<Vec<PltColor>> {
    let palette = colorous::categorical("plt_colorous_categorical", &name)?;
    Ok(Some(palette.iter().map(|c| colorous::to_color(*c)).collect()))
}
