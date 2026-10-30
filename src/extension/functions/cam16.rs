// ============================================================================
// CAM16 的自定义观察条件（palette 的 `cam16::Parameters` 及其使用）。
//
// `plt_color` 里的 `cam16_*` 空间始终使用 palette 的默认条件；这里的函数让观察条件成为
// 参数：
//   - `plt_cam16_params(white_point, La, Yb, surround[, discounting])`：白点是名字；
//     `plt_cam16_params_xyz(x, y, z, La, Yb, surround[, discounting])`：白点是数值
//     （palette 的动态白点）。省略 discounting 即 auto；
//   - `plt_cam16_from_xyz` / `plt_cam16_to_xyz`：`xyz` ↔ 全属性 `plt_cam16_full`；
//   - `plt_cam16_partial_from_xyz` / `plt_cam16_partial_to_xyz`：`xyz` ↔ 八个部分空间
//     （`cam16_jch` … `cam16_ucs_jmh`）。
//
// 输入必须是 `xyz` 空间，分量被当作「该观察条件下白点的三刺激值」；返回的 `xyz` 同理。
// 要从任意空间进出，先用 `plt_convert(color, 'xyz')` / 再 `plt_convert(xyz, target)`。
//
// ---------------------------------------------------------------------------
// CAM16 with explicit viewing conditions. `plt_cam16_params*` builds the five
// `Parameters` fields (named or numeric white point, auto or custom discounting); the
// `plt_cam16_*_from_xyz` / `*_to_xyz` pairs convert between `xyz` and the full or partial CAM16
// types. Convert other spaces to `xyz` with `plt_convert` first.
// ============================================================================

use duckfn::{DuckOptionResult, duck_scalar_function};

use crate::extension::types::cam16::{self, CAM16_SPACES, PltCam16Params, SURROUNDS};
use crate::extension::types::color::{PltCam16Full, PltColor};

/// 默认观察条件：D65、La=40、Yb=0.2、average、auto。
#[duck_scalar_function(
    description = "palette's default CAM16 viewing conditions (D65, La = 40, Yb = 0.2, average surround, auto discounting)",
    example = "SELECT plt_cam16_default_params()"
)]
fn plt_cam16_default_params() -> PltCam16Params {
    PltCam16Params::defaults()
}

/// 周边条件的预设名。
#[duck_scalar_function(
    description = "The named CAM16 surround presets (dark, dim, average) and their percentages",
    example = "SELECT plt_cam16_surrounds()"
)]
fn plt_cam16_surrounds() -> Vec<String> {
    SURROUNDS.iter().map(|s| s.to_string()).collect()
}

/// 自定义观察条件，白点按名字，折扣 auto。
#[duck_scalar_function(
    overloads_name = "plt_cam16_params",
    description = "CAM16 viewing conditions from a named white point (auto discounting); surround is dark, dim, average or a percentage 0..20",
    example = "SELECT plt_cam16_params('d65', 40, 0.2, 'average')"
)]
fn plt_cam16_params(
    white_point: String,
    adapting_luminance: f64,
    background_luminance: f64,
    surround: String,
) -> DuckOptionResult<PltCam16Params> {
    Ok(Some(cam16::params_named(
        "plt_cam16_params",
        &white_point,
        adapting_luminance,
        background_luminance,
        &surround,
        None,
    )?))
}

/// 自定义观察条件，白点按名字，折扣给定（0..1）。
#[duck_scalar_function(
    overloads_name = "plt_cam16_params",
    description = "CAM16 viewing conditions from a named white point with a custom discounting factor (0..1), instead of auto",
    example = "SELECT plt_cam16_params('d65', 40, 0.2, 'dim', 0.9)"
)]
fn plt_cam16_params_custom(
    white_point: String,
    adapting_luminance: f64,
    background_luminance: f64,
    surround: String,
    discounting: f64,
) -> DuckOptionResult<PltCam16Params> {
    Ok(Some(cam16::params_named(
        "plt_cam16_params",
        &white_point,
        adapting_luminance,
        background_luminance,
        &surround,
        Some(discounting),
    )?))
}

/// 自定义观察条件，白点按 XYZ 数值，折扣 auto。
#[duck_scalar_function(
    overloads_name = "plt_cam16_params_xyz",
    description = "CAM16 viewing conditions from a custom white point XYZ (auto discounting)",
    example = "SELECT plt_cam16_params_xyz(0.95047, 1, 1.08883, 40, 0.2, 'average')"
)]
fn plt_cam16_params_xyz(
    white_x: f64,
    white_y: f64,
    white_z: f64,
    adapting_luminance: f64,
    background_luminance: f64,
    surround: String,
) -> DuckOptionResult<PltCam16Params> {
    Ok(Some(cam16::params(
        "plt_cam16_params_xyz",
        (white_x, white_y, white_z),
        adapting_luminance,
        background_luminance,
        &surround,
        None,
    )?))
}

/// 自定义观察条件，白点按 XYZ 数值，折扣给定（0..1）。
#[duck_scalar_function(
    overloads_name = "plt_cam16_params_xyz",
    description = "CAM16 viewing conditions from a custom white point XYZ with a custom discounting factor (0..1)",
    example = "SELECT plt_cam16_params_xyz(0.95047, 1, 1.08883, 40, 0.2, 'average', 0.8)"
)]
fn plt_cam16_params_xyz_custom(
    white_x: f64,
    white_y: f64,
    white_z: f64,
    adapting_luminance: f64,
    background_luminance: f64,
    surround: String,
    discounting: f64,
) -> DuckOptionResult<PltCam16Params> {
    Ok(Some(cam16::params(
        "plt_cam16_params_xyz",
        (white_x, white_y, white_z),
        adapting_luminance,
        background_luminance,
        &surround,
        Some(discounting),
    )?))
}

/// `xyz` → CAM16 全属性（六个量：J、C、h、Q、M、s）。
#[duck_scalar_function(
    description = "Convert an xyz color into the full CAM16 attributes (lightness, chroma, hue, brightness, colorfulness, saturation) under custom viewing conditions",
    example = "SELECT plt_cam16_from_xyz(plt_xyz(0.4, 0.3, 0.2), plt_cam16_default_params())"
)]
fn plt_cam16_from_xyz(color: PltColor, params: PltCam16Params) -> DuckOptionResult<PltCam16Full> {
    let cam16 = cam16::full_from_xyz(&color, "plt_cam16_from_xyz", &params)?;
    let mut full = PltCam16Full::from(cam16);
    full.alpha = color.alpha;
    Ok(Some(full))
}

/// CAM16 全属性 → `xyz`。
#[duck_scalar_function(
    description = "Convert the full CAM16 attributes back into an xyz color under custom viewing conditions",
    example = "SELECT plt_cam16_to_xyz(plt_cam16_from_xyz(plt_xyz(0.4, 0.3, 0.2), plt_cam16_default_params()), plt_cam16_default_params())"
)]
fn plt_cam16_to_xyz(cam16: PltCam16Full, params: PltCam16Params) -> DuckOptionResult<PltColor> {
    let mut color = cam16::full_to_xyz(cam16.to_cam16(), &params);
    color.alpha = cam16.alpha;
    Ok(Some(color))
}

/// `xyz` → 某个 CAM16 部分空间，使用自定义观察条件。
#[duck_scalar_function(
    description = "Convert an xyz color into a partial CAM16 space (cam16_jch, cam16_jmh, cam16_jsh, cam16_qch, cam16_qmh, cam16_qsh, cam16_ucs_jab, cam16_ucs_jmh) under custom viewing conditions",
    example = "SELECT plt_cam16_partial_from_xyz(plt_xyz(0.4, 0.3, 0.2), 'cam16_jch', plt_cam16_default_params())"
)]
fn plt_cam16_partial_from_xyz(
    color: PltColor,
    space: String,
    params: PltCam16Params,
) -> DuckOptionResult<PltColor> {
    Ok(Some(cam16::partial_from_xyz(
        &color,
        "plt_cam16_partial_from_xyz",
        &space,
        &params,
    )?))
}

/// 某个 CAM16 部分空间 → `xyz`，使用自定义观察条件。
#[duck_scalar_function(
    description = "Convert a partial CAM16 color (cam16_jch etc.) back into an xyz color under custom viewing conditions",
    example = "SELECT plt_cam16_partial_to_xyz(plt_cam16_jch(50, 40, 120), plt_cam16_default_params())"
)]
fn plt_cam16_partial_to_xyz(color: PltColor, params: PltCam16Params) -> DuckOptionResult<PltColor> {
    Ok(Some(cam16::partial_to_xyz(
        &color,
        "plt_cam16_partial_to_xyz",
        &params,
    )?))
}

/// 列出支持自定义观察条件的 CAM16 空间。
#[duck_scalar_function(
    description = "Every CAM16 space that can be produced with custom viewing conditions",
    example = "SELECT plt_cam16_spaces()"
)]
fn plt_cam16_spaces() -> Vec<String> {
    CAM16_SPACES.iter().map(|s| s.to_string()).collect()
}
