// ============================================================================
// 3x3 矩阵运算（palette 的 `matrix` 模块与 `convert::Matrix3`）。
//
// 矩阵一律用 `LIST(DOUBLE)` 的 9 个数表示，行主序：
//
//     [m11, m12, m13, m21, m22, m23, m31, m32, m33]
//
// `plt_matrix_multiply(a, b)` 是「先 a 后 b」（palette 的 `Matrix3::then`），
// `plt_apply_matrix(matrix, color)` 把矩阵作用到颜色当前空间的三个分量上（不换空间）。
// `matrix_map` 这类逐元素映射在 SQL 里用 `list_transform` 表达，不单独包装。
//
// Matrices are nine doubles in row-major order. `plt_matrix_multiply(a, b)` applies `a` first and
// then `b`; `plt_apply_matrix` applies a matrix to the three components in the color's own space.
// Element-wise mapping is left to SQL's `list_transform`.
// ============================================================================

use duckfn::{DuckOptionResult, duck_scalar_function};

use crate::extension::types::color::PltColor;
use crate::extension::types::matrix;

/// 单位矩阵。
#[duck_scalar_function(
    description = "The 3x3 identity matrix, row-major as LIST(DOUBLE) of 9",
    example = "SELECT plt_matrix_identity()"
)]
fn plt_matrix_identity() -> Vec<f64> {
    matrix::matrix_list(matrix::identity())
}

/// 对角缩放矩阵。
#[duck_scalar_function(
    description = "A diagonal 3x3 matrix with the given diagonal entries, row-major",
    example = "SELECT plt_matrix_scale(0.5, 0.5, 0.5)"
)]
fn plt_matrix_scale(s1: f64, s2: f64, s3: f64) -> Vec<f64> {
    matrix::matrix_list(matrix::scale(s1, s2, s3))
}

/// 矩阵乘法（先 a 后 b）。
#[duck_scalar_function(
    description = "Multiply two 3x3 matrices, applying the first and then the second (row-major LIST(DOUBLE) of 9)",
    example = "SELECT plt_matrix_multiply(plt_matrix_identity(), plt_matrix_scale(2, 2, 2))"
)]
fn plt_matrix_multiply(a: Vec<f64>, b: Vec<f64>) -> DuckOptionResult<Vec<f64>> {
    let a = matrix::matrix_arg("plt_matrix_multiply", a)?;
    let b = matrix::matrix_arg("plt_matrix_multiply", b)?;
    Ok(Some(matrix::matrix_list(matrix::multiply(a, b))))
}

/// 矩阵求逆。
#[duck_scalar_function(
    description = "Invert a 3x3 matrix (row-major LIST(DOUBLE) of 9)",
    example = "SELECT plt_matrix_inverse(plt_matrix_scale(2, 4, 0.5))"
)]
fn plt_matrix_inverse(values: Vec<f64>) -> DuckOptionResult<Vec<f64>> {
    let values = matrix::matrix_arg("plt_matrix_inverse", values)?;
    Ok(Some(matrix::matrix_list(matrix::inverse(values))))
}

/// 矩阵作用到三分量向量。
#[duck_scalar_function(
    description = "Apply a 3x3 matrix to three numbers, returning LIST(DOUBLE) of 3",
    example = "SELECT plt_matrix_apply(plt_matrix_identity(), 1, 2, 3)"
)]
fn plt_matrix_apply(values: Vec<f64>, x: f64, y: f64, z: f64) -> DuckOptionResult<Vec<f64>> {
    let values = matrix::matrix_arg("plt_matrix_apply", values)?;
    let (x, y, z) = matrix::apply(values, (x, y, z));
    Ok(Some(vec![x, y, z]))
}

/// 矩阵作用到颜色的三个分量上（不换空间）。
#[duck_scalar_function(
    description = "Apply a 3x3 matrix to a color's three components in its current space (the space tag is kept)",
    example = "SELECT plt_apply_matrix(plt_adaptation_matrix('d65', 'd50', 'bradford'), plt_xyz(0.5, 0.4, 0.3))"
)]
fn plt_apply_matrix(values: Vec<f64>, color: PltColor) -> DuckOptionResult<PltColor> {
    let values = matrix::matrix_arg("plt_apply_matrix", values)?;
    let space = color.require_space("plt_apply_matrix")?;
    let (c1, c2, c3) = matrix::apply(values, (color.c1, color.c2, color.c3));
    Ok(Some(color.with_components(space, c1, c2, c3)))
}
