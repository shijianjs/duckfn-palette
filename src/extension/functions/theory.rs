// ============================================================================
// 传统色彩理论（palette 的 `color_theory` 的标量面）。
//
// 这些方案只在带色相的空间里有定义（hsl / hsv / hwb / hsluv / lch / lchuv / oklch / okhsl /
// okhsv / okhwb / 6 个 CAM16 部分空间 / cam16_ucs_jmh），其他空间报查询错误并提示先换空间。
// 每个函数返回「除输入色之外的其余颜色」，顺序与 palette 一致；色相归一到 0..360。
//
// Color theory schemes for hue spaces. Each function returns the scheme's other colors, in
// palette's order, with hues normalized to 0..360.
// ============================================================================

use duckfn::{DuckOptionResult, duck_scalar_function};

use crate::extension::types::color::PltColor;
use crate::extension::types::theory::SCHEMES;

/// 互补色（色相 +180°）。
#[duck_scalar_function(
    description = "The complementary color (hue + 180 degrees) of a color; the color must have a hue",
    example = "SELECT plt_complementary(plt_hsl(120, 1, 0.5))"
)]
fn plt_complementary(color: PltColor) -> DuckOptionResult<PltColor> {
    let mut colors = color.harmony("complementary", "plt_complementary")?;
    Ok(colors.pop())
}

/// 两个分裂互补色（色相 +150°、+210°）。
#[duck_scalar_function(
    description = "The two split complementary colors (hue + 150 and + 210 degrees) of a color",
    example = "SELECT plt_split_complementary(plt_hsl(120, 1, 0.5))"
)]
fn plt_split_complementary(color: PltColor) -> DuckOptionResult<Vec<PltColor>> {
    Ok(Some(
        color.harmony("split_complementary", "plt_split_complementary")?,
    ))
}

/// 两个邻近色（色相 -30°、+30°）。
#[duck_scalar_function(
    description = "The two analogous colors (hue - 30 and + 30 degrees) of a color",
    example = "SELECT plt_analogous(plt_hsl(120, 1, 0.5))"
)]
fn plt_analogous(color: PltColor) -> DuckOptionResult<Vec<PltColor>> {
    Ok(Some(color.harmony("analogous", "plt_analogous")?))
}

/// 再外一层的两个邻近色（色相 -60°、+60°）。
#[duck_scalar_function(
    description = "The next two analogous colors (hue - 60 and + 60 degrees) of a color",
    example = "SELECT plt_analogous_secondary(plt_hsl(120, 1, 0.5))"
)]
fn plt_analogous_secondary(color: PltColor) -> DuckOptionResult<Vec<PltColor>> {
    Ok(Some(
        color.harmony("analogous_secondary", "plt_analogous_secondary")?,
    ))
}

/// 两个三角色（色相 +120°、+240°）。
#[duck_scalar_function(
    description = "The two triadic colors (hue + 120 and + 240 degrees) of a color",
    example = "SELECT plt_triadic(plt_hsl(120, 1, 0.5))"
)]
fn plt_triadic(color: PltColor) -> DuckOptionResult<Vec<PltColor>> {
    Ok(Some(color.harmony("triadic", "plt_triadic")?))
}

/// 三个四方色（色相 +90°、+180°、+270°）。
#[duck_scalar_function(
    description = "The three tetradic (square) colors (hue + 90, + 180 and + 270 degrees) of a color",
    example = "SELECT plt_tetradic(plt_hsl(120, 1, 0.5))"
)]
fn plt_tetradic(color: PltColor) -> DuckOptionResult<Vec<PltColor>> {
    Ok(Some(color.harmony("tetradic", "plt_tetradic")?))
}

/// 通用入口：按方案名取其余颜色。
#[duck_scalar_function(
    description = "The other colors of a color theory scheme by name (complementary, split_complementary, analogous, analogous_secondary, triadic, tetradic)",
    example = "SELECT plt_harmony(plt_oklch(0.7, 0.1, 120), 'triadic')"
)]
fn plt_harmony(color: PltColor, scheme: String) -> DuckOptionResult<Vec<PltColor>> {
    Ok(Some(color.harmony(&scheme, "plt_harmony")?))
}

/// 所有方案名。
#[duck_scalar_function(
    description = "All supported color theory scheme names, as accepted by plt_harmony",
    example = "SELECT plt_color_schemes()"
)]
fn plt_color_schemes() -> Vec<String> {
    SCHEMES.iter().map(|s| s.to_string()).collect()
}
