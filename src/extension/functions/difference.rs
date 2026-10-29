// ============================================================================
// 色差与对比度（palette 的 `color_difference` 与 `relative_contrast` 的标量面）。
//
// 色差有七种度量，每种只在特定空间里有定义，所以 `plt_difference` 带一个可选的空间参数
// （3 参重载用默认的 CIE L*a*b*，4 参重载显式指定）。对比度按 WCAG 2.1，内部转到 sRGB 计算。
//
// Color differences with seven metrics and an optional space argument, plus WCAG 2.1 contrast.
// ============================================================================

use duckfn::{DuckOptionResult, duck_scalar_function};

use crate::extension::types::color::PltColor;
use crate::extension::types::difference::{CONTRAST_CRITERIA, METRICS};

/// 色差（默认在 CIE L*a*b* 里算）。
#[duck_scalar_function(
    overloads_name = "plt_difference",
    description = "Color difference between two colors with a metric (euclidean, euclidean_squared, delta_e, improved_delta_e, hyab, ciede2000, improved_ciede2000), computed in CIE L*a*b*",
    example = "SELECT plt_difference(plt_lab(50, 2.6772, -79.7751), plt_lab(50, 0, -82.7485), 'ciede2000')"
)]
fn plt_difference_3(a: PltColor, b: PltColor, metric: String) -> DuckOptionResult<f64> {
    Ok(Some(a.difference(&b, &metric, None, "plt_difference")?))
}

/// 色差（显式指定空间）。
#[duck_scalar_function(
    overloads_name = "plt_difference",
    description = "Color difference between two colors with a metric, computed in the given color space",
    example = "SELECT plt_difference(plt_srgb(1, 0, 0), plt_srgb(0, 1, 0), 'euclidean', 'oklab')"
)]
fn plt_difference_4(
    a: PltColor,
    b: PltColor,
    metric: String,
    space: String,
) -> DuckOptionResult<f64> {
    Ok(Some(a.difference(&b, &metric, Some(&space), "plt_difference")?))
}

/// 所有色差度量名。
#[duck_scalar_function(
    description = "All supported color difference metric names, as accepted by plt_difference",
    example = "SELECT plt_difference_metrics()"
)]
fn plt_difference_metrics() -> Vec<String> {
    METRICS.iter().map(|m| m.to_string()).collect()
}

/// WCAG 2.1 相对亮度。
#[duck_scalar_function(
    description = "WCAG 2.1 relative luminance of a color (0 is black, 1 is white), computed by converting to sRGB",
    example = "SELECT plt_relative_luminance(plt_srgb(0.5, 0.5, 0.5))"
)]
fn plt_relative_luminance(color: PltColor) -> DuckOptionResult<f64> {
    Ok(Some(color.relative_luminance("plt_relative_luminance")?))
}

/// WCAG 2.1 相对对比度。
#[duck_scalar_function(
    description = "WCAG 2.1 relative contrast ratio between two colors (1 to 21), computed by converting both to sRGB",
    example = "SELECT plt_contrast(plt_srgb(1, 1, 1), plt_srgb(0, 0, 0))"
)]
fn plt_contrast(a: PltColor, b: PltColor) -> DuckOptionResult<f64> {
    Ok(Some(a.contrast_ratio(&b, "plt_contrast")?))
}

/// 是否满足某条 WCAG 2.1 对比度判定。
#[duck_scalar_function(
    description = "Whether two colors meet a WCAG 2.1 contrast criterion (min_text, min_large_text, enhanced_text, enhanced_large_text, min_graphics)",
    example = "SELECT plt_contrast_check(plt_srgb(1, 1, 1), plt_srgb(0, 0, 0), 'min_text')"
)]
fn plt_contrast_check(a: PltColor, b: PltColor, criterion: String) -> DuckOptionResult<bool> {
    Ok(Some(a.contrast_meets(&b, &criterion, "plt_contrast_check")?))
}

/// 所有对比度判定名。
#[duck_scalar_function(
    description = "All supported WCAG 2.1 contrast criterion names, as accepted by plt_contrast_check",
    example = "SELECT plt_contrast_criteria()"
)]
fn plt_contrast_criteria() -> Vec<String> {
    CONTRAST_CRITERIA.iter().map(|c| c.to_string()).collect()
}
