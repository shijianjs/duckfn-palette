// ============================================================================
// 3x3 矩阵（palette 的 `matrix` 模块与 `convert::Matrix3`）。
//
// SQL 用 `LIST(DOUBLE)` 的 9 个数表示矩阵，行主序（与 palette 的 `Mat3` 布局一致）：
//
//     [m11, m12, m13, m21, m22, m23, m31, m32, m33]
//
// 这里是纯函数层：参数校验、矩阵运算与作用到向量/颜色分量。
//
// 3x3 matrices as nine doubles in row-major order, matching palette's `Mat3`. Pure helpers:
// argument checking, the palette matrix operations, and applying a matrix to a vector.
// ============================================================================

use palette::matrix::{matrix_inverse, multiply_3x3, multiply_3x3_and_vec3};
use quack_rs::error::ExtensionError;

use super::color::bad_arg;

/// 把 SQL 传入的 LIST(DOUBLE) 校验成 9 元素矩阵。
///
/// Validate a nine-element matrix from SQL.
pub fn matrix_arg(fn_name: &str, values: Vec<f64>) -> Result<[f64; 9], ExtensionError> {
    <[f64; 9]>::try_from(values).map_err(|values: Vec<f64>| {
        bad_arg(
            fn_name,
            format!(
                "expected a LIST(DOUBLE) with exactly 9 matrix elements (row-major), got {}",
                values.len()
            ),
        )
    })
}

/// 矩阵 → `LIST(DOUBLE)`。
///
/// A matrix as a nine-element list.
pub fn matrix_list(matrix: [f64; 9]) -> Vec<f64> {
    matrix.to_vec()
}

/// 单位矩阵。
///
/// The identity matrix.
pub fn identity() -> [f64; 9] {
    [1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0]
}

/// 对角缩放矩阵。
///
/// A diagonal scale matrix.
pub fn scale(s1: f64, s2: f64, s3: f64) -> [f64; 9] {
    [s1, 0.0, 0.0, 0.0, s2, 0.0, 0.0, 0.0, s3]
}

/// 矩阵乘法（先 `a` 后 `b`，即 palette 的 `Matrix3::then`）。
///
/// Matrix product, applying `a` first and then `b` (palette's `Matrix3::then`).
pub fn multiply(a: [f64; 9], b: [f64; 9]) -> [f64; 9] {
    multiply_3x3(b, a)
}

/// 矩阵求逆。
///
/// Matrix inverse.
pub fn inverse(matrix: [f64; 9]) -> [f64; 9] {
    matrix_inverse(matrix)
}

/// 矩阵作用到三分量向量。
///
/// Apply a matrix to a three-component vector.
pub fn apply(matrix: [f64; 9], vector: (f64, f64, f64)) -> (f64, f64, f64) {
    let [x, y, z] = multiply_3x3_and_vec3(matrix, [vector.0, vector.1, vector.2]);
    (x, y, z)
}
