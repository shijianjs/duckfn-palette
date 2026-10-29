// ============================================================================
// 混合与合成（palette 的 `Blend` / `Compose` / `BlendWith` + `Equations` 的标量面）。
//
// 与 palette 一致：第一个参数是源（source），第二个是背景（destination）；第二个颜色若不在
// 第一个颜色的空间里，会先转过去。`Blend` 的 11 个模式只在 RGB / Luma / Lms / Xyz 上有定义；
// `Compose` 与 `plt_blend_equations` 还覆盖 Lab / Luv / Oklab / Yxy / Cam16UcsJab。
//
// Blending and composition. The first argument is the source and the second the destination, as in
// palette; a second color in another space is converted first. Blend modes need an RGB-like
// stimulus space; Compose and custom equations also cover Lab/Luv/Oklab/Yxy/Cam16UcsJab.
// ============================================================================

use duckfn::{DuckOptionResult, duck_scalar_function};

use crate::extension::types::blend::{BLEND_MODES, BLEND_PARAMETERS, COMPOSE_MODES};
use crate::extension::types::color::PltColor;

/// 11 种可分离混合模式。
#[duck_scalar_function(
    description = "Blend a source color onto a destination color with a separable mode (multiply, screen, overlay, darken, lighten, dodge, burn, hard_light, soft_light, difference, exclusion)",
    example = "SELECT plt_blend(plt_srgb(0.5, 0.2, 0.1), plt_srgb(0.4, 0.6, 0.8), 'multiply')"
)]
fn plt_blend(a: PltColor, b: PltColor, mode: String) -> DuckOptionResult<PltColor> {
    Ok(Some(a.blend(&b, &mode, "plt_blend")?))
}

/// 6 种 Porter-Duff 合成操作。
#[duck_scalar_function(
    description = "Composite a source color with a destination color using a Porter-Duff operator (over, inside, outside, atop, xor, plus)",
    example = "SELECT plt_compose(plt_with_alpha(plt_srgb(1, 0, 0), 0.5), plt_with_alpha(plt_srgb(0, 0, 1), 0.5), 'over')"
)]
fn plt_compose(a: PltColor, b: PltColor, mode: String) -> DuckOptionResult<PltColor> {
    Ok(Some(a.compose(&b, &mode, "plt_compose")?))
}

/// OpenGL 风格的自定义混合因子。
#[duck_scalar_function(
    description = "Blend a source color onto a destination color with OpenGL-style source/destination factors (zero, one, source_color, one_minus_source_color, destination_color, one_minus_destination_color, source_alpha, one_minus_source_alpha, destination_alpha, one_minus_destination_alpha)",
    example = "SELECT plt_blend_equations(plt_with_alpha(plt_srgb(1, 0, 0), 0.5), plt_with_alpha(plt_srgb(0, 0, 1), 0.5), 'source_alpha', 'one_minus_source_alpha')"
)]
fn plt_blend_equations(
    a: PltColor,
    b: PltColor,
    source: String,
    destination: String,
) -> DuckOptionResult<PltColor> {
    Ok(Some(a.blend_equations(
        &b,
        &source,
        &destination,
        "plt_blend_equations",
    )?))
}

/// 所有混合模式名。
#[duck_scalar_function(
    description = "All supported blend mode names, as accepted by plt_blend",
    example = "SELECT plt_blend_modes()"
)]
fn plt_blend_modes() -> Vec<String> {
    BLEND_MODES.iter().map(|m| m.to_string()).collect()
}

/// 所有合成模式名。
#[duck_scalar_function(
    description = "All supported composition operator names, as accepted by plt_compose",
    example = "SELECT plt_compose_modes()"
)]
fn plt_compose_modes() -> Vec<String> {
    COMPOSE_MODES.iter().map(|m| m.to_string()).collect()
}

/// 所有混合因子名。
#[duck_scalar_function(
    description = "All supported blend factor names, as accepted by plt_blend_equations",
    example = "SELECT plt_blend_parameters()"
)]
fn plt_blend_parameters() -> Vec<String> {
    BLEND_PARAMETERS.iter().map(|m| m.to_string()).collect()
}
