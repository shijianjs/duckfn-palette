// ============================================================================
// 颜色操作族（palette 的 `Mix` / `Lighten` / `Darken` / `Saturate` / `Desaturate` /
// `ShiftHue` / `WithHue` / `GetHue` / `Premultiply` 的标量面）。
//
// 所有操作都在颜色自己所在的空间里执行，不隐式换空间；空间不支持该操作时报查询错误，消息里
// 建议先用 `plt_convert` 换到支持它的空间。`plt_mix` 例外：第二个操作数会被转到第一个操作数的
// 空间，alpha 一并线性插值。
//
// The manipulation family. Every operation runs in the color's own space (no implicit
// conversion); unsupported spaces raise a query error that suggests `plt_convert` first.
// `plt_mix` converts its second operand into the first one's space and interpolates alpha too.
// ============================================================================

use duckfn::{DuckOptionResult, duck_scalar_function};

use crate::extension::types::color::PltColor;

/// 在同一空间里按 factor 线性插值（0 = 第一个，1 = 第二个）。
#[duck_scalar_function(
    description = "Mix two colors by a factor (0 = first, 1 = second) in the first color's space; alpha is mixed too",
    example = "SELECT plt_mix(plt_oklch(0.6, 0.1, 30), plt_oklch(0.8, 0.1, 200), 0.5)"
)]
fn plt_mix(a: PltColor, b: PltColor, factor: f64) -> DuckOptionResult<PltColor> {
    Ok(Some(a.mix_with(&b, factor, "plt_mix")?))
}

/// 朝最大亮度按比例提亮。
#[duck_scalar_function(
    description = "Lighten a color by scaling its lightness towards the maximum (factor 0..1)",
    example = "SELECT plt_lighten(plt_hsl(0, 1, 0.5), 0.5)"
)]
fn plt_lighten(color: PltColor, factor: f64) -> DuckOptionResult<PltColor> {
    Ok(Some(color.lighten(factor, "plt_lighten")?))
}

/// 按绝对量提亮。
#[duck_scalar_function(
    description = "Lighten a color by an absolute amount of its lightness range (amount 0..1)",
    example = "SELECT plt_lighten_fixed(plt_hsl(0, 1, 0.4), 0.2)"
)]
fn plt_lighten_fixed(color: PltColor, amount: f64) -> DuckOptionResult<PltColor> {
    Ok(Some(color.lighten_fixed(amount, "plt_lighten_fixed")?))
}

/// 朝最小亮度按比例压暗。
#[duck_scalar_function(
    description = "Darken a color by scaling its lightness towards the minimum (factor 0..1)",
    example = "SELECT plt_darken(plt_hsv(0, 1, 0.5), 0.5)"
)]
fn plt_darken(color: PltColor, factor: f64) -> DuckOptionResult<PltColor> {
    Ok(Some(color.darken(factor, "plt_darken")?))
}

/// 按绝对量压暗。
#[duck_scalar_function(
    description = "Darken a color by an absolute amount of its lightness range (amount 0..1)",
    example = "SELECT plt_darken_fixed(plt_hsv(0, 1, 0.4), 0.2)"
)]
fn plt_darken_fixed(color: PltColor, amount: f64) -> DuckOptionResult<PltColor> {
    Ok(Some(color.darken_fixed(amount, "plt_darken_fixed")?))
}

/// 按比例提高饱和度/彩度。
#[duck_scalar_function(
    description = "Increase saturation or chroma by scaling towards the maximum (factor 0..1)",
    example = "SELECT plt_saturate(plt_hsl(0, 0.5, 0.5), 0.5)"
)]
fn plt_saturate(color: PltColor, factor: f64) -> DuckOptionResult<PltColor> {
    Ok(Some(color.saturate(factor, "plt_saturate")?))
}

/// 按绝对量提高饱和度/彩度。
#[duck_scalar_function(
    description = "Increase saturation or chroma by an absolute amount (amount 0..1)",
    example = "SELECT plt_saturate_fixed(plt_hsl(0, 0.4, 0.5), 0.2)"
)]
fn plt_saturate_fixed(color: PltColor, amount: f64) -> DuckOptionResult<PltColor> {
    Ok(Some(color.saturate_fixed(amount, "plt_saturate_fixed")?))
}

/// 按比例降低饱和度/彩度。
#[duck_scalar_function(
    description = "Decrease saturation or chroma by scaling towards the minimum (factor 0..1)",
    example = "SELECT plt_desaturate(plt_hsl(0, 0.5, 0.5), 0.5)"
)]
fn plt_desaturate(color: PltColor, factor: f64) -> DuckOptionResult<PltColor> {
    Ok(Some(color.desaturate(factor, "plt_desaturate")?))
}

/// 按绝对量降低饱和度/彩度。
#[duck_scalar_function(
    description = "Decrease saturation or chroma by an absolute amount (amount 0..1)",
    example = "SELECT plt_desaturate_fixed(plt_hsl(0, 0.4, 0.5), 0.2)"
)]
fn plt_desaturate_fixed(color: PltColor, amount: f64) -> DuckOptionResult<PltColor> {
    Ok(Some(
        color.desaturate_fixed(amount, "plt_desaturate_fixed")?,
    ))
}

/// 色相位移（度）。
#[duck_scalar_function(
    description = "Rotate the hue of a color by degrees (any real number; wraps at 360)",
    example = "SELECT plt_shift_hue(plt_hsl(120, 1, 0.5), 120)"
)]
fn plt_shift_hue(color: PltColor, degrees: f64) -> DuckOptionResult<PltColor> {
    Ok(Some(color.shift_hue(degrees, "plt_shift_hue")?))
}

/// 把色相设为给定角度。
#[duck_scalar_function(
    description = "Set the hue of a color to the given degrees",
    example = "SELECT plt_with_hue(plt_hsl(120, 1, 0.5), 240)"
)]
fn plt_with_hue(color: PltColor, degrees: f64) -> DuckOptionResult<PltColor> {
    Ok(Some(color.with_hue(degrees, "plt_with_hue")?))
}

/// 读出色相角（度）。
#[duck_scalar_function(
    description = "The hue of a color in degrees (0..360); gray colors report 0",
    example = "SELECT plt_hue(plt_srgb(1, 0, 0))"
)]
fn plt_hue(color: PltColor) -> DuckOptionResult<f64> {
    Ok(Some(color.get_hue("plt_hue")?))
}

/// 分量乘以 alpha（预乘 alpha）。
#[duck_scalar_function(
    description = "Premultiply a color by its alpha (components multiplied by alpha; alpha must be in 0..1)",
    example = "SELECT plt_premultiply(plt_with_alpha(plt_srgb(1, 0.5, 0.25), 0.5))"
)]
fn plt_premultiply(color: PltColor) -> DuckOptionResult<PltColor> {
    Ok(Some(color.premultiply("plt_premultiply")?))
}

/// 分量除以 alpha（撤销预乘）。
#[duck_scalar_function(
    description = "Undo premultiplication (components divided by alpha; zero alpha yields zero components)",
    example = "SELECT plt_unpremultiply(plt_premultiply(plt_with_alpha(plt_srgb(1, 0.5, 0.25), 0.5)))"
)]
fn plt_unpremultiply(color: PltColor) -> DuckOptionResult<PltColor> {
    Ok(Some(color.unpremultiply("plt_unpremultiply")?))
}

/// 在每个 sRGB 通道上加上 amount（0..1），保持空间与 alpha。
#[duck_scalar_function(
    description = "Brighten by adding an amount (0..1 of the channel range) to every sRGB channel, clamped, and converting back to the input space",
    example = "SELECT plt_brighten(plt_srgb(0.2, 0.3, 0.4), 0.1)"
)]
fn plt_brighten(color: PltColor, amount: f64) -> DuckOptionResult<PltColor> {
    Ok(Some(color.brighten(amount, "plt_brighten")?))
}

/// 把饱和度降为 0（经 HSL），保持空间与 alpha。
#[duck_scalar_function(
    description = "Greyscale a color by dropping its HSL saturation to zero, returning to the input space",
    example = "SELECT plt_greyscale(plt_srgb(1, 0, 0))"
)]
fn plt_greyscale(color: PltColor) -> DuckOptionResult<PltColor> {
    Ok(Some(color.greyscale("plt_greyscale")?))
}

/// 朝白色混合 amount（0..1），保持空间与 alpha。
#[duck_scalar_function(
    description = "Tint a color by mixing it towards white in sRGB by amount (0 unchanged, 1 white); alpha is unchanged",
    example = "SELECT plt_tint(plt_srgb(1, 0, 0), 0.5)"
)]
fn plt_tint(color: PltColor, amount: f64) -> DuckOptionResult<PltColor> {
    Ok(Some(color.tint(amount, "plt_tint")?))
}

/// 朝黑色混合 amount（0..1），保持空间与 alpha。
#[duck_scalar_function(
    description = "Shade a color by mixing it towards black in sRGB by amount (0 unchanged, 1 black); alpha is unchanged",
    example = "SELECT plt_shade(plt_srgb(1, 0, 0), 0.5)"
)]
fn plt_shade(color: PltColor, amount: f64) -> DuckOptionResult<PltColor> {
    Ok(Some(color.shade(amount, "plt_shade")?))
}
