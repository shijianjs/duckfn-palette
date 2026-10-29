// ============================================================================
// 白点与色适应（palette 的 `white_point` / `chromatic_adaptation` / `lms::matrix`）。
//
// 白点在 palette 里是类型参数，SQL 里是名字：`plt_white_points()` 列出全部，`plt_white_point_xyz`
// / `plt_white_point_xy` 给出数值。色适应三件套：
//   - `plt_adaptation_matrix(source, target, method)`：XYZ(source) → XYZ(target) 的完整矩阵；
//   - `plt_lms_matrix` / `plt_lms_to_xyz_matrix` / `plt_diagonal_adaptation_matrix`：palette 把
//     适应拆开时暴露的零件，供自己拼更复杂的矩阵；
//   - `plt_chromatic_adapt(color, source, target, method)`：直接把矩阵作用到 `xyz` 颜色上。
//
// `plt_chromatic_adapt` 的输入必须是 `xyz` 空间：分量被当作「source 白点下的 XYZ 三刺激值」，
// 返回的颜色仍标 `xyz`，但数值已在 target 白点下 —— 白点不在值里，需要调用方自己记住
// （palette 用类型参数承载它，SQL 里没有这个位置）。
//
// White points by name and chromatic adaptation. `plt_chromatic_adapt` takes a color in `xyz` whose
// components are tristimulus values under the source white point and returns the same tag with
// values under the target white point; the white point itself is not part of the value.
// ============================================================================

use duckfn::{DuckOptionResult, duck_scalar_function};

use crate::extension::types::adaptation;
use crate::extension::types::color::PltColor;
use crate::extension::types::matrix;

/// 全部白点名。
#[duck_scalar_function(
    description = "Every supported white point name (CIE illuminants plus the DCI-P3 theatre white)",
    example = "SELECT plt_white_points()"
)]
fn plt_white_points() -> Vec<String> {
    adaptation::WHITE_POINTS
        .iter()
        .map(|w| w.to_string())
        .collect()
}

/// 白点的 XYZ 三刺激值。
#[duck_scalar_function(
    description = "The tristimulus values (X, Y, Z) of a white point, as LIST(DOUBLE) of 3 with Y = 1",
    example = "SELECT plt_white_point_xyz('d65')"
)]
fn plt_white_point_xyz(name: String) -> DuckOptionResult<Vec<f64>> {
    let (x, y, z) = adaptation::white_point_xyz("plt_white_point_xyz", &name)?;
    Ok(Some(vec![x, y, z]))
}

/// 白点的 xy 色度。
#[duck_scalar_function(
    description = "The chromaticity coordinates (x, y) of a white point, as LIST(DOUBLE) of 2",
    example = "SELECT plt_white_point_xy('d50')"
)]
fn plt_white_point_xy(name: String) -> DuckOptionResult<Vec<f64>> {
    let (x, y, z) = adaptation::white_point_xyz("plt_white_point_xy", &name)?;
    let sum = x + y + z;
    Ok(Some(vec![x / sum, y / sum]))
}

/// 全部色适应方法。
#[duck_scalar_function(
    description = "Every supported chromatic adaptation method (bradford, von_kries, unit_matrix)",
    example = "SELECT plt_adaptation_methods()"
)]
fn plt_adaptation_methods() -> Vec<String> {
    adaptation::ADAPTATION_METHODS
        .iter()
        .map(|m| m.to_string())
        .collect()
}

/// 色适应矩阵：XYZ(source) → XYZ(target)。
#[duck_scalar_function(
    description = "The 3x3 matrix adapting XYZ values from a source white point to a target one, row-major",
    example = "SELECT plt_adaptation_matrix('a', 'c', 'bradford')"
)]
fn plt_adaptation_matrix(
    source: String,
    target: String,
    method: String,
) -> DuckOptionResult<Vec<f64>> {
    Ok(Some(matrix::matrix_list(adaptation::adaptation_matrix(
        "plt_adaptation_matrix",
        &source,
        &target,
        &method,
    )?)))
}

/// XYZ→LMS 矩阵（色适应方法的零件）。
#[duck_scalar_function(
    description = "The XYZ-to-LMS matrix used by an adaptation method, row-major",
    example = "SELECT plt_lms_matrix('bradford')"
)]
fn plt_lms_matrix(method: String) -> DuckOptionResult<Vec<f64>> {
    Ok(Some(matrix::matrix_list(adaptation::lms_matrix(
        "plt_lms_matrix",
        &method,
    )?)))
}

/// LMS→XYZ 矩阵（色适应方法的零件）。
#[duck_scalar_function(
    description = "The LMS-to-XYZ matrix used by an adaptation method, row-major",
    example = "SELECT plt_lms_to_xyz_matrix('bradford')"
)]
fn plt_lms_to_xyz_matrix(method: String) -> DuckOptionResult<Vec<f64>> {
    Ok(Some(matrix::matrix_list(adaptation::lms_to_xyz_matrix(
        "plt_lms_to_xyz_matrix",
        &method,
    )?)))
}

/// LMS 空间的增益对角矩阵（palette 的 `diagonal_matrix`）。
#[duck_scalar_function(
    description = "The diagonal LMS gain matrix between two white points, to be used between plt_lms_matrix and plt_lms_to_xyz_matrix",
    example = "SELECT plt_diagonal_adaptation_matrix('a', 'c', 'bradford')"
)]
fn plt_diagonal_adaptation_matrix(
    source: String,
    target: String,
    method: String,
) -> DuckOptionResult<Vec<f64>> {
    Ok(Some(matrix::matrix_list(
        adaptation::diagonal_adaptation_matrix(
            "plt_diagonal_adaptation_matrix",
            &source,
            &target,
            &method,
        )?,
    )))
}

/// 把 `xyz` 颜色从 D65 适应到目标白点。
#[duck_scalar_function(
    overloads_name = "plt_chromatic_adapt",
    description = "Adapt an xyz color from D65 to another white point (returns xyz values under the target white point)",
    example = "SELECT plt_chromatic_adapt(plt_xyz(0.5, 0.4, 0.3), 'd50', 'bradford')"
)]
fn plt_chromatic_adapt_3(
    color: PltColor,
    target: String,
    method: String,
) -> DuckOptionResult<PltColor> {
    Ok(Some(adaptation::adapt_xyz(
        "plt_chromatic_adapt",
        &color,
        "d65",
        &target,
        &method,
    )?))
}

/// 把 `xyz` 颜色从给定白点适应到目标白点。
#[duck_scalar_function(
    overloads_name = "plt_chromatic_adapt",
    description = "Adapt an xyz color from a source white point to a target white point (the color's components are XYZ under the source)",
    example = "SELECT plt_chromatic_adapt(plt_xyz(0.315756, 0.162732, 0.015905), 'a', 'c', 'bradford')"
)]
fn plt_chromatic_adapt_4(
    color: PltColor,
    source: String,
    target: String,
    method: String,
) -> DuckOptionResult<PltColor> {
    Ok(Some(adaptation::adapt_xyz(
        "plt_chromatic_adapt",
        &color,
        &source,
        &target,
        &method,
    )?))
}
