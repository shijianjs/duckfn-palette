// ============================================================================
// 色适应与白点（palette 的 `chromatic_adaptation` / `white_point` / `lms::matrix`）。
//
// palette 把白点做成类型参数，SQL 里换成白点名（a / b / c / d50 / d55 / d65 / d75 / e /
// f2 / f7 / f11 / 10° 变体 / dci_p3）。色适应矩阵沿用 palette 的公式：
//
//     M = M_lms→xyz · diag(out_lms / in_lms) · M_xyz→lms
//
// `M_xyz→lms` / `M_lms→xyz` 由方法决定（bradford / von_kries / unit_matrix），白点的 XYZ 来自
// `WhitePoint::get_xyz()`。palette 的泛型 `adaptation_matrix::<T, I, O, M>` 需要把 15×15 个白点
// 类型对逐一单态化才能运行时分派，这里改为用 palette 公开的矩阵函数与白点数值按同一条公式计算，
// 结果与它一致（官方示例 A→C 的期望值见测试）。
//
// `plt_chromatic_adapt` 只接受 `xyz` 空间的颜色：分量即「在 source 白点下的 XYZ 三刺激值」，
// 返回的颜色仍标 `xyz`，但三刺激值改到 target 白点下 —— 白点不在值里，必须由调用方自己记住。
//
// White points and chromatic adaptation. The adaptation matrix follows palette's formula with
// palette's own LMS matrices and white-point values, since palette's generic helper would need
// 15×15 type pairs monomorphized for runtime dispatch. `plt_chromatic_adapt` takes a color in `xyz`
// (components are tristimulus values under the source white point) and returns the same tag with
// values under the target white point.
// ============================================================================

use palette::encoding::DciP3;
use palette::lms::matrix::{Bradford, LmsToXyz, UnitMatrix, VonKries, XyzToLms};
use palette::matrix::{multiply_3x3, multiply_3x3_and_vec3, Mat3};
use palette::white_point::{
    WhitePoint, A, B, C, D50, D55, D65, D75, D50Degree10, D55Degree10, D65Degree10, D75Degree10,
    E, F11, F2, F7,
};
use quack_rs::error::ExtensionError;

use super::color::{PltColor, bad_arg};

/// 所有可用的白点名（规范形式）。
///
/// Every supported white point name, normalized.
pub const WHITE_POINTS: &[&str] = &[
    "a",
    "b",
    "c",
    "d50",
    "d55",
    "d65",
    "d75",
    "e",
    "f2",
    "f7",
    "f11",
    "d50_degree10",
    "d55_degree10",
    "d65_degree10",
    "d75_degree10",
    "dci_p3",
];

/// 所有可用的色适应方法（规范形式）。
///
/// Every supported chromatic adaptation method, normalized.
pub const ADAPTATION_METHODS: &[&str] = &["bradford", "von_kries", "unit_matrix"];

/// 名字规范化：小写，`-` 与空格视作 `_`。
///
/// Normalize a name: lowercase, `-` and spaces become `_`.
fn normalize(name: &str) -> String {
    name.trim().to_ascii_lowercase().replace(['-', ' '], "_")
}

/// 白点名解析（含 10° 变体与常见别名的规范化）。
///
/// Resolve a white point name, normalizing the 10° variants and a few aliases.
pub fn white_point_arg(fn_name: &str, name: &str) -> Result<&'static str, ExtensionError> {
    let normalized = normalize(name);
    let canonical = match normalized.as_str() {
        "d50_10" | "d50_10deg" | "d50_degree_10" => "d50_degree10",
        "d55_10" | "d55_10deg" | "d55_degree_10" => "d55_degree10",
        "d65_10" | "d65_10deg" | "d65_degree_10" => "d65_degree10",
        "d75_10" | "d75_10deg" | "d75_degree_10" => "d75_degree10",
        "illuminant_a" => "a",
        "illuminant_b" => "b",
        "illuminant_c" => "c",
        "illuminant_e" => "e",
        other => other,
    };
    WHITE_POINTS
        .iter()
        .copied()
        .find(|w| *w == canonical)
        .ok_or_else(|| {
            bad_arg(
                fn_name,
                format!(
                    "unknown white point '{name}'; supported: {}",
                    WHITE_POINTS.join(", ")
                ),
            )
        })
}

/// 白点的 XYZ 三刺激值（Y 已归一化为 1）。
///
/// The white point's tristimulus values (Y normalized to 1).
pub fn white_point_xyz(
    fn_name: &str,
    name: &str,
) -> Result<(f64, f64, f64), ExtensionError> {
    let xyz = match white_point_arg(fn_name, name)? {
        "a" => A::get_xyz(),
        "b" => B::get_xyz(),
        "c" => C::get_xyz(),
        "d50" => D50::get_xyz(),
        "d55" => D55::get_xyz(),
        "d65" => D65::get_xyz(),
        "d75" => D75::get_xyz(),
        "e" => E::get_xyz(),
        "f2" => F2::get_xyz(),
        "f7" => F7::get_xyz(),
        "f11" => F11::get_xyz(),
        "d50_degree10" => D50Degree10::get_xyz(),
        "d55_degree10" => D55Degree10::get_xyz(),
        "d65_degree10" => D65Degree10::get_xyz(),
        "d75_degree10" => D75Degree10::get_xyz(),
        "dci_p3" => <DciP3 as WhitePoint<f64>>::get_xyz(),
        _ => unreachable!("white_point_arg returned an unknown white point"),
    };
    Ok(xyz.into_components())
}

/// 色适应方法解析。
///
/// Resolve an adaptation method name.
pub fn method_arg(fn_name: &str, method: &str) -> Result<&'static str, ExtensionError> {
    let normalized = normalize(method);
    let canonical = match normalized.as_str() {
        "vonkries" | "hpe" | "hunt_pointer_estevez" => "von_kries",
        "unit" | "identity" | "none" => "unit_matrix",
        other => other,
    };
    ADAPTATION_METHODS
        .iter()
        .copied()
        .find(|m| *m == canonical)
        .ok_or_else(|| {
            bad_arg(
                fn_name,
                format!(
                    "unknown adaptation method '{method}'; supported: {}",
                    ADAPTATION_METHODS.join(", ")
                ),
            )
        })
}

/// 方法对应的「XYZ→LMS」与「LMS→XYZ」矩阵。
///
/// The XYZ→LMS and LMS→XYZ matrices of an adaptation method.
fn lms_matrices(
    fn_name: &str,
    method: &str,
) -> Result<(Mat3<f64>, Mat3<f64>), ExtensionError> {
    Ok(match method_arg(fn_name, method)? {
        "bradford" => (
            Bradford::xyz_to_lms_matrix(),
            Bradford::lms_to_xyz_matrix(),
        ),
        "von_kries" => (
            VonKries::xyz_to_lms_matrix(),
            VonKries::lms_to_xyz_matrix(),
        ),
        "unit_matrix" => (
            UnitMatrix::xyz_to_lms_matrix(),
            UnitMatrix::lms_to_xyz_matrix(),
        ),
        _ => unreachable!("method_arg returned an unknown method"),
    })
}

/// `XyzToLms` 的矩阵本身（供用户自己拼矩阵）。
///
/// The XYZ→LMS matrix itself.
pub fn lms_matrix(fn_name: &str, method: &str) -> Result<[f64; 9], ExtensionError> {
    Ok(lms_matrices(fn_name, method)?.0)
}

/// `LmsToXyz` 的矩阵本身。
///
/// The LMS→XYZ matrix itself.
pub fn lms_to_xyz_matrix(fn_name: &str, method: &str) -> Result<[f64; 9], ExtensionError> {
    Ok(lms_matrices(fn_name, method)?.1)
}

/// 两个白点在 LMS 空间的增益比（palette 的 `diagonal_matrix`）。
///
/// The LMS gain ratio between two white points (palette's `diagonal_matrix`).
fn lms_gains(
    fn_name: &str,
    source: &str,
    target: &str,
    method: &str,
) -> Result<[f64; 9], ExtensionError> {
    let (to_lms, _) = lms_matrices(fn_name, method)?;
    let source = white_point_xyz(fn_name, source)?;
    let target = white_point_xyz(fn_name, target)?;
    let source = multiply_3x3_and_vec3(to_lms, [source.0, source.1, source.2]);
    let target = multiply_3x3_and_vec3(to_lms, [target.0, target.1, target.2]);
    Ok([
        target[0] / source[0],
        0.0,
        0.0,
        0.0,
        target[1] / source[1],
        0.0,
        0.0,
        0.0,
        target[2] / source[2],
    ])
}

/// 完整的色适应矩阵：XYZ(source) → XYZ(target)。
///
/// The full adaptation matrix from XYZ under `source` to XYZ under `target`.
pub fn adaptation_matrix(
    fn_name: &str,
    source: &str,
    target: &str,
    method: &str,
) -> Result<[f64; 9], ExtensionError> {
    let (to_lms, from_lms) = lms_matrices(fn_name, method)?;
    let gain = lms_gains(fn_name, source, target, method)?;
    Ok(multiply_3x3(from_lms, multiply_3x3(gain, to_lms)))
}

/// LMS 空间的增益矩阵（palette 的 `diagonal_matrix`，需要在 LMS 空间里使用）。
///
/// The LMS-space gain matrix (palette's `diagonal_matrix`); apply it between the LMS matrices.
pub fn diagonal_adaptation_matrix(
    fn_name: &str,
    source: &str,
    target: &str,
    method: &str,
) -> Result<[f64; 9], ExtensionError> {
    lms_gains(fn_name, source, target, method)
}

/// 把一个 `xyz` 颜色从 source 白点适应到 target 白点。
///
/// Adapt an `xyz` color from the source white point to the target one.
pub fn adapt_xyz(
    fn_name: &str,
    color: &PltColor,
    source: &str,
    target: &str,
    method: &str,
) -> Result<PltColor, ExtensionError> {
    if color.require_space(fn_name)? != "xyz" {
        return Err(bad_arg(
            fn_name,
            format!(
                "expected a color in the 'xyz' space (its components are XYZ tristimulus values \
                 under the source white point), got '{}'; convert it first, e.g. \
                 plt_convert(color, 'xyz')",
                color.space
            ),
        ));
    }
    let matrix = adaptation_matrix(fn_name, source, target, method)?;
    let [x, y, z] = multiply_3x3_and_vec3(matrix, [color.c1, color.c2, color.c3]);
    Ok(PltColor::new("xyz", x, y, z, color.alpha))
}
