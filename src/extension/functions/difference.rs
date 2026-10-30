// ============================================================================
// 色差与对比度（palette 的 `color_difference` 与 `relative_contrast` 的标量面）。
//
// 色差有七种度量，每种只在特定空间里有定义，所以 `plt_difference` 带一个可选的空间参数
// （3 参重载用默认的 CIE L*a*b*，4 参重载显式指定）。对比度按 WCAG 2.1，内部转到 sRGB 计算。
//
// Color differences with seven metrics and an optional space argument, plus WCAG 2.1 contrast.
// ============================================================================

use duckfn::{DuckOptionResult, duck_scalar_function};
use quack_rs::error::ExtensionError;

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
    Ok(Some(a.difference(
        &b,
        &metric,
        Some(&space),
        "plt_difference",
    )?))
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

/// tinycolor 的「亮/暗」分界：YIQ 亮度 128（0..255 量程）。
///
/// tinycolor's light/dark threshold: YIQ brightness 128 on the 0..255 scale.
const YIQ_LIGHT_THRESHOLD: f64 = 128.0 / 255.0;

/// YIQ 亮度（0 黑 .. 1 白），按 sRGB 通道加权；分量先收进 0..1。
///
/// YIQ brightness (0 black .. 1 white) from the sRGB channels, clamped to 0..1 first.
fn yiq_brightness(color: &PltColor, fn_name: &str) -> Result<f64, ExtensionError> {
    let srgb = color.convert(fn_name, "srgb")?;
    let channel = |value: f64| value.clamp(0.0, 1.0);
    Ok(0.299 * channel(srgb.c1) + 0.587 * channel(srgb.c2) + 0.114 * channel(srgb.c3))
}

/// YIQ 亮度（0 黑 .. 1 白）。
#[duck_scalar_function(
    description = "YIQ brightness (0 black .. 1 white) from the sRGB channels, clamped to 0..1; alpha is ignored",
    example = "SELECT plt_brightness(plt_srgb(0.2, 0.6, 0.9))"
)]
fn plt_brightness(color: PltColor) -> DuckOptionResult<f64> {
    Ok(Some(yiq_brightness(&color, "plt_brightness")?))
}

/// 是否属于「亮色」（YIQ 亮度 >= 128/255）。
#[duck_scalar_function(
    description = "Whether a color is light by tinycolor's YIQ rule (brightness >= 128/255); alpha is ignored",
    example = "SELECT plt_is_light(plt_srgb(1, 1, 1))"
)]
fn plt_is_light(color: PltColor) -> DuckOptionResult<bool> {
    Ok(Some(
        yiq_brightness(&color, "plt_is_light")? >= YIQ_LIGHT_THRESHOLD,
    ))
}

/// 是否属于「暗色」（YIQ 亮度 < 128/255）。
#[duck_scalar_function(
    description = "Whether a color is dark by tinycolor's YIQ rule (brightness < 128/255); alpha is ignored",
    example = "SELECT plt_is_dark(plt_srgb(0, 0, 0))"
)]
fn plt_is_dark(color: PltColor) -> DuckOptionResult<bool> {
    Ok(Some(
        yiq_brightness(&color, "plt_is_dark")? < YIQ_LIGHT_THRESHOLD,
    ))
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
    Ok(Some(a.contrast_meets(
        &b,
        &criterion,
        "plt_contrast_check",
    )?))
}

/// 所有对比度判定名。
#[duck_scalar_function(
    description = "All supported WCAG 2.1 contrast criterion names, as accepted by plt_contrast_check",
    example = "SELECT plt_contrast_criteria()"
)]
fn plt_contrast_criteria() -> Vec<String> {
    CONTRAST_CRITERIA.iter().map(|c| c.to_string()).collect()
}

/// 从候选色里挑对比度最高的一个（默认判定 min_text，不回退）。
#[duck_scalar_function(
    overloads_name = "plt_most_readable",
    description = "The candidate with the highest WCAG contrast ratio against the base color; no black/white fallback",
    example = "SELECT plt_most_readable(plt_srgb(1, 1, 1), [plt_srgb(0, 0, 0), plt_srgb(0.9, 0.9, 0.9)])"
)]
fn plt_most_readable_2(base: PltColor, candidates: Vec<PltColor>) -> DuckOptionResult<PltColor> {
    Ok(Some(PltColor::most_readable(
        &base,
        &candidates,
        "min_text",
        false,
        "plt_most_readable",
    )?))
}

/// 带判定标准的候选色选择。
#[duck_scalar_function(
    overloads_name = "plt_most_readable",
    description = "The candidate with the highest WCAG contrast ratio against the base color, using the given criterion for the optional fallback decision",
    example = "SELECT plt_most_readable(plt_srgb(1, 1, 1), [plt_srgb(0.9, 0.9, 0.9)], 'min_text')"
)]
fn plt_most_readable_3(
    base: PltColor,
    candidates: Vec<PltColor>,
    criterion: String,
) -> DuckOptionResult<PltColor> {
    Ok(Some(PltColor::most_readable(
        &base,
        &candidates,
        &criterion,
        false,
        "plt_most_readable",
    )?))
}

/// 带判定标准与黑/白回退的候选色选择。
#[duck_scalar_function(
    overloads_name = "plt_most_readable",
    description = "The candidate with the highest WCAG contrast ratio against the base color; when none meets the criterion and include_fallback is true, the more contrasting of black and white instead",
    example = "SELECT plt_most_readable(plt_srgb(1, 1, 1), [plt_srgb(0.9, 0.9, 0.9)], 'min_text', true)"
)]
fn plt_most_readable_4(
    base: PltColor,
    candidates: Vec<PltColor>,
    criterion: String,
    include_fallback: bool,
) -> DuckOptionResult<PltColor> {
    Ok(Some(PltColor::most_readable(
        &base,
        &candidates,
        &criterion,
        include_fallback,
        "plt_most_readable",
    )?))
}
