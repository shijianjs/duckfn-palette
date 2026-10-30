// ============================================================================
// 编码信息（palette 的 `encoding` 模块 + `RgbSpace` / `Primaries` 的只读面）。
//
// 这一组回答「这个空间的颜色是怎么编码的」：传输函数是什么、单个值怎么解码/编码、白点是谁、
// 原色在哪、RGB↔XYZ 矩阵是什么。它们不改变颜色值本身；换空间仍用 `plt_convert`。
//
// Read-only encoding information: which transfer function a space uses, decoding/encoding a single
// value, the space's white point and RGB primaries, and the RGB↔XYZ matrices. Nothing here changes
// a color value; use `plt_convert` for that.
// ============================================================================

use duckfn::{DuckOptionResult, duck_scalar_function};

use crate::extension::types::color::PltColor;
use crate::extension::types::encoding;
use crate::extension::types::matrix;

/// 有传输函数的空间名。
#[duck_scalar_function(
    description = "Spaces that have a transfer function (RGB spaces plus luma), as accepted by plt_decode and plt_encode",
    example = "SELECT plt_encoding_spaces()"
)]
fn plt_encoding_spaces() -> Vec<String> {
    encoding::ENCODING_SPACES
        .iter()
        .map(|s| s.to_string())
        .collect()
}

/// 空间使用的传输函数名。
#[duck_scalar_function(
    description = "The transfer function a space uses (linear, gamma_2_2, srgb, adobe_rgb, prophoto_rgb, p3_gamma or rec_oetf)",
    example = "SELECT plt_transfer_function('srgb')"
)]
fn plt_transfer_function(space: String) -> DuckOptionResult<String> {
    Ok(Some(
        encoding::transfer_function("plt_transfer_function", &space)?.to_string(),
    ))
}

/// 全部传输函数名。
#[duck_scalar_function(
    description = "Every transfer function name used by the supported spaces",
    example = "SELECT plt_transfer_functions()"
)]
fn plt_transfer_functions() -> Vec<String> {
    encoding::TRANSFER_FUNCTIONS
        .iter()
        .map(|f| f.to_string())
        .collect()
}

/// 单个编码值 → 线性值。
#[duck_scalar_function(
    description = "Decode one encoded component value into linear light using the space's transfer function",
    example = "SELECT plt_decode('srgb', 0.5)"
)]
fn plt_decode(space: String, value: f64) -> DuckOptionResult<f64> {
    Ok(Some(encoding::decode("plt_decode", &space, value)?))
}

/// 单个线性值 → 编码值。
#[duck_scalar_function(
    description = "Encode one linear component value with the space's transfer function",
    example = "SELECT plt_encode('srgb', 0.21404114)"
)]
fn plt_encode(space: String, value: f64) -> DuckOptionResult<f64> {
    Ok(Some(encoding::encode("plt_encode", &space, value)?))
}

/// 空间的参考白点名。
#[duck_scalar_function(
    description = "The white point a color space is defined against (e.g. d65 for srgb, d50 for prophoto_rgb)",
    example = "SELECT plt_white_point_of('prophoto_rgb')"
)]
fn plt_white_point_of(space: String) -> DuckOptionResult<String> {
    Ok(Some(
        encoding::white_point_of("plt_white_point_of", &space)?.to_string(),
    ))
}

/// RGB 空间的三原色（yxy 颜色，红绿蓝各一个）。
#[duck_scalar_function(
    description = "The red, green and blue primaries of an RGB space, as three yxy colors",
    example = "SELECT plt_rgb_primaries('srgb')"
)]
fn plt_rgb_primaries(space: String) -> DuckOptionResult<Vec<PltColor>> {
    Ok(Some(encoding::primaries("plt_rgb_primaries", &space)?))
}

/// RGB→XYZ 矩阵（行主序 9 个数）。
#[duck_scalar_function(
    description = "The 3x3 RGB-to-XYZ matrix of an RGB space, row-major as LIST(DOUBLE) of 9",
    example = "SELECT plt_rgb_to_xyz_matrix('srgb')"
)]
fn plt_rgb_to_xyz_matrix(space: String) -> DuckOptionResult<Vec<f64>> {
    Ok(Some(matrix::matrix_list(encoding::rgb_to_xyz(
        "plt_rgb_to_xyz_matrix",
        &space,
    )?)))
}

/// XYZ→RGB 矩阵（行主序 9 个数）。
#[duck_scalar_function(
    description = "The 3x3 XYZ-to-RGB matrix of an RGB space (the inverse of plt_rgb_to_xyz_matrix), row-major",
    example = "SELECT plt_xyz_to_rgb_matrix('srgb')"
)]
fn plt_xyz_to_rgb_matrix(space: String) -> DuckOptionResult<Vec<f64>> {
    Ok(Some(matrix::matrix_list(encoding::xyz_to_rgb(
        "plt_xyz_to_rgb_matrix",
        &space,
    )?)))
}
