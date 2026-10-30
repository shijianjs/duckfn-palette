// ============================================================================
// palette 的颜色构造、读写与转换（`convert` / `rgb` / `alpha` / `hues` 的标量面）。
//
// 形状：
//   - 每个颜色空间一个构造器（`plt_srgb`、`plt_lab`、…），参数顺序与 palette 的 `new` 一致，
//     分量单位照 palette：RGB/HSL/HSV/HWB/OK 系列 0..1，Lab/Luv/HSLuv 的 L 0..100，
//     所有 hue 用角度（0..360，允许负值与超过 360 的值，做色相运算时按 palette 的规则取模）；
//   - `plt_color(space, c1, c2, c3 [, alpha])` 是运行时版本的通用构造器；
//   - `plt_convert(color, space)` 走 palette 的 `FromColorUnclamped`，不裁剪 gamut；
//     `plt_convert_clamped` 再按目标空间的 `Clamp` 收进标准范围；
//   - alpha 由 `PltColor` 独立持有，转换不会改动它；`plt_with_alpha` / `plt_opaque` 显式设置。
//
// Construction, accessors and conversion. One constructor per palette color space, a runtime
// `plt_color`, and unclamped/clamped conversion; alpha rides along untouched.
// ============================================================================

use duckfn::{DuckOptionResult, duck_scalar_function};

use crate::extension::functions::err;
use crate::extension::types::color::{PltColor, SPACES, space_arg};

// ----------------------------------------------------------------------------
// 通用构造器与读写
// ----------------------------------------------------------------------------

/// 运行时构造器（无 alpha，默认 1.0）。
#[duck_scalar_function(
    overloads_name = "plt_color",
    description = "Build a color in any supported space from a space name and three components (alpha defaults to 1)",
    example = "SELECT plt_color('lab', 50, 20, -30)"
)]
fn plt_color_3(space: String, c1: f64, c2: f64, c3: f64) -> DuckOptionResult<PltColor> {
    let space = space_arg("plt_color", &space)?;
    Ok(Some(PltColor::new(space, c1, c2, c3, 1.0)))
}

/// 运行时构造器（显式 alpha）。
#[duck_scalar_function(
    overloads_name = "plt_color",
    description = "Build a color in any supported space from a space name, three components and an alpha value",
    example = "SELECT plt_color('oklch', 0.7, 0.1, 120, 0.5)"
)]
fn plt_color_4(space: String, c1: f64, c2: f64, c3: f64, alpha: f64) -> DuckOptionResult<PltColor> {
    let space = space_arg("plt_color", &space)?;
    Ok(Some(PltColor::new(space, c1, c2, c3, alpha)))
}

/// 颜色值的空间名。
#[duck_scalar_function(
    description = "The color space name of a color value (as returned by plt_color_spaces)",
    example = "SELECT plt_space(plt_srgb(1, 0, 0))"
)]
fn plt_space(color: PltColor) -> DuckOptionResult<String> {
    let space = color.require_space("plt_space")?;
    Ok(Some(space.to_string()))
}

/// 三个分量，按空间的自然顺序。
#[duck_scalar_function(
    description = "The three components of a color as a LIST(DOUBLE), in the space's natural order",
    example = "SELECT plt_components(plt_srgb(0.1, 0.2, 0.3))"
)]
fn plt_components(color: PltColor) -> Vec<f64> {
    vec![color.c1, color.c2, color.c3]
}

/// 单个分量（1 起算）。
#[duck_scalar_function(
    description = "One component of a color by 1-based index (1..3)",
    example = "SELECT plt_component(plt_srgb(0.1, 0.2, 0.3), 2)"
)]
fn plt_component(color: PltColor, index: i64) -> DuckOptionResult<f64> {
    let component = match index {
        1 => color.c1,
        2 => color.c2,
        3 => color.c3,
        _ => {
            return Err(err(format!(
                "plt_component: index must be 1, 2 or 3, got {index}"
            )));
        }
    };
    Ok(Some(component))
}

/// alpha 通道。
#[duck_scalar_function(
    description = "The alpha channel of a color (0 transparent, 1 opaque)",
    example = "SELECT plt_alpha(plt_color('srgb', 1, 0, 0, 0.25))"
)]
fn plt_alpha(color: PltColor) -> f64 {
    color.alpha
}

/// 设置 alpha，其余不变。
#[duck_scalar_function(
    description = "Set the alpha channel of a color, leaving space and components unchanged",
    example = "SELECT plt_alpha(plt_with_alpha(plt_srgb(1, 0, 0), 0.5))"
)]
fn plt_with_alpha(color: PltColor, alpha: f64) -> PltColor {
    PltColor::new(&color.space, color.c1, color.c2, color.c3, alpha)
}

/// 把 alpha 设为 1。
#[duck_scalar_function(
    description = "Make a color fully opaque (alpha = 1)",
    example = "SELECT plt_alpha(plt_opaque(plt_color('srgb', 1, 0, 0, 0.25)))"
)]
fn plt_opaque(color: PltColor) -> PltColor {
    PltColor::new(&color.space, color.c1, color.c2, color.c3, 1.0)
}

/// 两个颜色是否渲染成同一份 8 位 sRGB（跨空间比较，alpha 两位小数）。
#[duck_scalar_function(
    description = "Whether two colors render as the same 8-bit sRGB value and alpha (rounded to two decimals), regardless of their spaces",
    example = "SELECT plt_equals(plt_srgb(1, 0, 0), plt_hsl(0, 1, 0.5))"
)]
fn plt_equals(a: PltColor, b: PltColor) -> DuckOptionResult<bool> {
    Ok(Some(a.same_rgba8(&b, "plt_equals")?))
}

/// 所有受支持的空间名。
#[duck_scalar_function(
    description = "All supported color space names, as accepted by plt_convert and plt_color",
    example = "SELECT plt_color_spaces()"
)]
fn plt_color_spaces() -> Vec<String> {
    SPACES.iter().map(|s| s.to_string()).collect()
}

/// 从 LIST(DOUBLE) 还原三个分量（alpha 默认 1）。
#[duck_scalar_function(
    description = "Build a color from a space name and a LIST(DOUBLE) of exactly three components",
    example = "SELECT plt_from_components('lab', [50, 20, -30])"
)]
fn plt_from_components(space: String, components: Vec<f64>) -> DuckOptionResult<PltColor> {
    let space = space_arg("plt_from_components", &space)?;
    if components.len() != 3 {
        return Err(err(format!(
            "plt_from_components: expected exactly 3 components, got {}",
            components.len()
        )));
    }
    Ok(Some(PltColor::new(
        space,
        components[0],
        components[1],
        components[2],
        1.0,
    )))
}

/// 从 LIST(DOUBLE) 还原四个分量（含 alpha）。
#[duck_scalar_function(
    description = "Build a color from a space name and a LIST(DOUBLE) of exactly four values (three components plus alpha)",
    example = "SELECT plt_from_components_alpha('lab', [50, 20, -30, 0.5])"
)]
fn plt_from_components_alpha(space: String, components: Vec<f64>) -> DuckOptionResult<PltColor> {
    let space = space_arg("plt_from_components_alpha", &space)?;
    if components.len() != 4 {
        return Err(err(format!(
            "plt_from_components_alpha: expected exactly 4 values, got {}",
            components.len()
        )));
    }
    Ok(Some(PltColor::new(
        space,
        components[0],
        components[1],
        components[2],
        components[3],
    )))
}

// ----------------------------------------------------------------------------
// 转换
// ----------------------------------------------------------------------------

/// 转换到目标空间，保留 alpha，不裁剪 gamut。
#[duck_scalar_function(
    description = "Convert a color to another space without gamut clamping (palette's unclamped conversion)",
    example = "SELECT plt_components(plt_convert(plt_srgb(1, 0, 0), 'lab'))"
)]
fn plt_convert(color: PltColor, space: String) -> DuckOptionResult<PltColor> {
    Ok(Some(color.convert("plt_convert", &space)?))
}

/// 转换到目标空间，再按目标空间的边界裁剪。
#[duck_scalar_function(
    description = "Convert a color to another space, then clamp the result to that space's standard range",
    example = "SELECT plt_components(plt_convert_clamped(plt_lab(50, 200, 200), 'srgb'))"
)]
fn plt_convert_clamped(color: PltColor, space: String) -> DuckOptionResult<PltColor> {
    Ok(Some(color.convert_clamped("plt_convert_clamped", &space)?))
}

/// 按当前空间的边界裁剪分量。
#[duck_scalar_function(
    description = "Clamp a color's components to its own space's standard range",
    example = "SELECT plt_components(plt_clamp(plt_lab(50, 200, 200)))"
)]
fn plt_clamp(color: PltColor) -> DuckOptionResult<PltColor> {
    Ok(Some(color.clamped("plt_clamp")?))
}

/// 是否已在当前空间的边界内。
#[duck_scalar_function(
    description = "Whether all components of a color are already within its space's standard range",
    example = "SELECT plt_is_within_bounds(plt_lab(50, 200, 200))"
)]
fn plt_is_within_bounds(color: PltColor) -> DuckOptionResult<bool> {
    Ok(Some(color.is_within_bounds("plt_is_within_bounds")?))
}

// ----------------------------------------------------------------------------
// 每个空间的构造器。参数顺序与 palette 的 `Color::new` 完全一致。
//
// 只有参数个数不同（luma 一个），所以不合并 overload；名字就是空间名。
// One constructor per space, same argument order as palette's `new`; luma takes one component.
// ----------------------------------------------------------------------------

macro_rules! constructor3 {
    ($fn_name:ident, $space:literal, $desc:literal, $example:literal) => {
        #[duck_scalar_function(description = $desc, example = $example)]
        fn $fn_name(c1: f64, c2: f64, c3: f64) -> PltColor {
            PltColor::new($space, c1, c2, c3, 1.0)
        }
    };
}

constructor3!(
    plt_srgb,
    "srgb",
    "Gamma-encoded sRGB color: red, green, blue in 0..1",
    "SELECT plt_srgb(1, 0, 0)"
);
constructor3!(
    plt_srgb_linear,
    "srgb_linear",
    "Linear-light sRGB color: red, green, blue in 0..1",
    "SELECT plt_srgb_linear(1, 0, 0)"
);
constructor3!(
    plt_srgb_gamma,
    "srgb_gamma",
    "Gamma 2.2 encoded sRGB color (palette's deprecated GammaSrgb): red, green, blue in 0..1",
    "SELECT plt_srgb_gamma(1, 0, 0)"
);
constructor3!(
    plt_adobe_rgb,
    "adobe_rgb",
    "Adobe RGB (1998) color: red, green, blue in 0..1",
    "SELECT plt_adobe_rgb(1, 0, 0)"
);
constructor3!(
    plt_adobe_rgb_linear,
    "adobe_rgb_linear",
    "Linear-light Adobe RGB color: red, green, blue in 0..1",
    "SELECT plt_adobe_rgb_linear(1, 0, 0)"
);
constructor3!(
    plt_display_p3,
    "display_p3",
    "Display P3 color (D65, sRGB transfer function): red, green, blue in 0..1",
    "SELECT plt_display_p3(1, 0, 0)"
);
constructor3!(
    plt_display_p3_linear,
    "display_p3_linear",
    "Linear-light Display P3 color: red, green, blue in 0..1",
    "SELECT plt_display_p3_linear(1, 0, 0)"
);
constructor3!(
    plt_dci_p3,
    "dci_p3",
    "DCI-P3 color (DCI-P3 white point, gamma 2.6): red, green, blue in 0..1",
    "SELECT plt_dci_p3(1, 0, 0)"
);
constructor3!(
    plt_dci_p3_linear,
    "dci_p3_linear",
    "Linear-light DCI-P3 color: red, green, blue in 0..1",
    "SELECT plt_dci_p3_linear(1, 0, 0)"
);
constructor3!(
    plt_prophoto_rgb,
    "prophoto_rgb",
    "ProPhoto RGB color (D50 white point, gamma 1.8): red, green, blue in 0..1",
    "SELECT plt_prophoto_rgb(1, 0, 0)"
);
constructor3!(
    plt_prophoto_rgb_linear,
    "prophoto_rgb_linear",
    "Linear-light ProPhoto RGB color (D50 white point): red, green, blue in 0..1",
    "SELECT plt_prophoto_rgb_linear(1, 0, 0)"
);
constructor3!(
    plt_rec709,
    "rec709",
    "Rec. 709 color (sRGB primaries, Rec. 709 transfer function): red, green, blue in 0..1",
    "SELECT plt_rec709(1, 0, 0)"
);
constructor3!(
    plt_rec709_linear,
    "rec709_linear",
    "Linear-light Rec. 709 color: red, green, blue in 0..1",
    "SELECT plt_rec709_linear(1, 0, 0)"
);
constructor3!(
    plt_rec2020,
    "rec2020",
    "Rec. 2020 color (D65, Rec. 2020 transfer function): red, green, blue in 0..1",
    "SELECT plt_rec2020(1, 0, 0)"
);
constructor3!(
    plt_rec2020_linear,
    "rec2020_linear",
    "Linear-light Rec. 2020 color: red, green, blue in 0..1",
    "SELECT plt_rec2020_linear(1, 0, 0)"
);
constructor3!(
    plt_hsl,
    "hsl",
    "HSL color: hue in degrees, saturation 0..1, lightness 0..1 (sRGB primaries)",
    "SELECT plt_hsl(120, 0.5, 0.5)"
);
constructor3!(
    plt_hsv,
    "hsv",
    "HSV color: hue in degrees, saturation 0..1, value 0..1 (sRGB primaries)",
    "SELECT plt_hsv(120, 0.5, 0.5)"
);
constructor3!(
    plt_hwb,
    "hwb",
    "HWB color: hue in degrees, whiteness 0..1, blackness 0..1 (sRGB primaries)",
    "SELECT plt_hwb(120, 0.2, 0.2)"
);
constructor3!(
    plt_lab,
    "lab",
    "CIELAB color (D65): L* 0..100, a*, b* roughly -128..127",
    "SELECT plt_lab(50, 20, -30)"
);
constructor3!(
    plt_lch,
    "lch",
    "CIE LCH color (D65): L* 0..100, chroma >= 0, hue in degrees",
    "SELECT plt_lch(50, 40, 120)"
);
constructor3!(
    plt_luv,
    "luv",
    "CIELUV color (D65): L* 0..100, u*, v*",
    "SELECT plt_luv(50, 20, -30)"
);
constructor3!(
    plt_lchuv,
    "lchuv",
    "CIE LCHuv color (D65): L* 0..100, chroma >= 0, hue in degrees (same model as HSLuv)",
    "SELECT plt_lchuv(50, 40, 120)"
);
constructor3!(
    plt_hsluv,
    "hsluv",
    "HSLuv color (D65): hue in degrees, saturation 0..100, lightness 0..100",
    "SELECT plt_hsluv(120, 50, 50)"
);
constructor3!(
    plt_xyz,
    "xyz",
    "CIE XYZ color with D65 white point: X, Y, Z",
    "SELECT plt_xyz(0.5, 0.4, 0.3)"
);
constructor3!(
    plt_xyz_d50,
    "xyz_d50",
    "CIE XYZ color with D50 white point: X, Y, Z (conversions adapt to/from D65 with Bradford)",
    "SELECT plt_xyz_d50(0.5, 0.4, 0.3)"
);
constructor3!(
    plt_yxy,
    "yxy",
    "CIE xyY color (D65): chromaticity x, y and luminance Y",
    "SELECT plt_yxy(0.3, 0.3, 0.5)"
);
constructor3!(
    plt_oklab,
    "oklab",
    "Oklab color: lightness 0..1, a, b (nominally -1..1)",
    "SELECT plt_oklab(0.5, 0.1, 0.1)"
);
constructor3!(
    plt_oklch,
    "oklch",
    "Oklch color: lightness 0..1, chroma >= 0, hue in degrees",
    "SELECT plt_oklch(0.5, 0.1, 120)"
);
constructor3!(
    plt_okhsl,
    "okhsl",
    "Okhsl color: hue in degrees, saturation 0..1, lightness 0..1",
    "SELECT plt_okhsl(120, 0.5, 0.5)"
);
constructor3!(
    plt_okhsv,
    "okhsv",
    "Okhsv color: hue in degrees, saturation 0..1, value 0..1",
    "SELECT plt_okhsv(120, 0.5, 0.5)"
);
constructor3!(
    plt_okhwb,
    "okhwb",
    "Okhwb color: hue in degrees, whiteness 0..1, blackness 0..1",
    "SELECT plt_okhwb(120, 0.2, 0.2)"
);
constructor3!(
    plt_cam16_jch,
    "cam16_jch",
    "CAM16 (J, C, h) under palette's default viewing conditions: lightness, chroma, hue in degrees",
    "SELECT plt_cam16_jch(50, 40, 120)"
);
constructor3!(
    plt_cam16_jmh,
    "cam16_jmh",
    "CAM16 (J, M, h) under default viewing conditions: lightness, colorfulness, hue in degrees",
    "SELECT plt_cam16_jmh(50, 40, 120)"
);
constructor3!(
    plt_cam16_jsh,
    "cam16_jsh",
    "CAM16 (J, s, h) under default viewing conditions: lightness, saturation, hue in degrees",
    "SELECT plt_cam16_jsh(50, 40, 120)"
);
constructor3!(
    plt_cam16_qch,
    "cam16_qch",
    "CAM16 (Q, C, h) under default viewing conditions: brightness, chroma, hue in degrees",
    "SELECT plt_cam16_qch(50, 40, 120)"
);
constructor3!(
    plt_cam16_qmh,
    "cam16_qmh",
    "CAM16 (Q, M, h) under default viewing conditions: brightness, colorfulness, hue in degrees",
    "SELECT plt_cam16_qmh(50, 40, 120)"
);
constructor3!(
    plt_cam16_qsh,
    "cam16_qsh",
    "CAM16 (Q, s, h) under default viewing conditions: brightness, saturation, hue in degrees",
    "SELECT plt_cam16_qsh(50, 40, 120)"
);
constructor3!(
    plt_cam16_ucs_jab,
    "cam16_ucs_jab",
    "CAM16-UCS (J', a', b') under default viewing conditions",
    "SELECT plt_cam16_ucs_jab(50, 20, -30)"
);
constructor3!(
    plt_cam16_ucs_jmh,
    "cam16_ucs_jmh",
    "CAM16-UCS (J', M', h) under default viewing conditions: lightness, colorfulness, hue in degrees",
    "SELECT plt_cam16_ucs_jmh(50, 40, 120)"
);
constructor3!(
    plt_lms,
    "lms",
    "LMS cone response (Bradford matrix, D65): long, medium, short",
    "SELECT plt_lms(0.5, 0.4, 0.3)"
);
constructor3!(
    plt_lms_von_kries,
    "lms_von_kries",
    "LMS cone response (von Kries / Hunt-Pointer-Estevez matrix, D65): long, medium, short",
    "SELECT plt_lms_von_kries(0.5, 0.4, 0.3)"
);

/// 一维亮度空间（编码 sRGB luma）。
#[duck_scalar_function(
    description = "sRGB-encoded luminance (grayscale) color: one component in 0..1",
    example = "SELECT plt_luma(0.5)"
)]
fn plt_luma(luma: f64) -> PltColor {
    PltColor::new("luma", luma, 0.0, 0.0, 1.0)
}

/// 线性亮度空间。
#[duck_scalar_function(
    description = "Linear-light luminance (grayscale) color: one component in 0..1",
    example = "SELECT plt_luma_linear(0.5)"
)]
fn plt_luma_linear(luma: f64) -> PltColor {
    PltColor::new("luma_linear", luma, 0.0, 0.0, 1.0)
}

/// Gamma 2.2 亮度空间。
#[duck_scalar_function(
    description = "Gamma 2.2 encoded luminance (grayscale) color: one component in 0..1",
    example = "SELECT plt_luma_gamma(0.5)"
)]
fn plt_luma_gamma(luma: f64) -> PltColor {
    PltColor::new("luma_gamma", luma, 0.0, 0.0, 1.0)
}
