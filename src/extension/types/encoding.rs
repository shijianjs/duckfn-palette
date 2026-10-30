// ============================================================================
// 编码信息与传输函数（palette 的 `encoding` 模块）。
//
// palette 把 RGB 标准拆成「`RgbSpace`（原色 + 白点）+ 传输函数」两层类型，这里在运行时按
// 空间名分派：
//   - `transfer_function`：空间用哪个传输函数（srgb / rec_oetf / p3_gamma / gamma_2_2 / ...）；
//   - `decode` / `encode`：单个值在「编码值 ↔ 线性值」之间往返（`IntoLinear` / `FromLinear`），
//     这是 `plt_convert` 之外唯一按分量暴露传输函数的入口；
//   - `white_point_of`：空间的参考白点；
//   - `primaries`：RGB 空间的红/绿/蓝原色，以 yxy 颜色返回（与 palette 的 `Primaries` 同值）；
//   - `rgb_to_xyz_matrix` / `xyz_to_rgb_matrix`：RGB↔XYZ 的 3x3 矩阵（`rgb_to_xyz_matrix` 动态
//     计算，`xyz_to_rgb_matrix` 取其逆）。
//
// The `encoding` module surfaced by name: transfer functions (decode/encode a single value), the
// white point of each space, RGB primaries, and the RGB↔XYZ matrices. RGB spaces accept both the
// gamma-encoded and the linear name; luma spaces reuse the same transfer functions.
// ============================================================================

#![allow(deprecated)]

use palette::encoding::gamma::{F2p2, GammaFn};
use palette::encoding::linear::LinearFn;
use palette::encoding::{
    AdobeRgb, AdobeRgb as AdobeRgbEncoding, DciP3 as DciP3Space, DisplayP3 as DisplayP3Space,
    FromLinear, IntoLinear, P3Gamma, ProPhotoRgb, ProPhotoRgb as ProPhotoRgbEncoding, Rec2020,
    RecOetf, Srgb, Srgb as SrgbEncoding,
};
use palette::matrix::{matrix_inverse, rgb_to_xyz_matrix};
use palette::rgb::{Primaries, RgbSpace};
use quack_rs::error::ExtensionError;

use super::color::{PltColor, bad_arg, space_arg};

/// 有传输函数的空间：15 个 RGB 名字 + 3 个 luma 名字。
///
/// Spaces with a transfer function: the 15 RGB names plus the 3 luma names.
pub const ENCODING_SPACES: &[&str] = &[
    "srgb",
    "srgb_linear",
    "srgb_gamma",
    "adobe_rgb",
    "adobe_rgb_linear",
    "display_p3",
    "display_p3_linear",
    "dci_p3",
    "dci_p3_linear",
    "prophoto_rgb",
    "prophoto_rgb_linear",
    "rec709",
    "rec709_linear",
    "rec2020",
    "rec2020_linear",
    "luma",
    "luma_linear",
    "luma_gamma",
];

/// palette 的传输函数名（规范形式）。
///
/// The transfer function names, normalized.
pub const TRANSFER_FUNCTIONS: &[&str] = &[
    "linear",
    "gamma_2_2",
    "srgb",
    "adobe_rgb",
    "prophoto_rgb",
    "p3_gamma",
    "rec_oetf",
];

/// 空间的规范名 + 必须是「有传输函数」的空间。
///
/// Canonical space name, restricted to the spaces that have a transfer function.
fn encoding_space(fn_name: &str, space: &str) -> Result<&'static str, ExtensionError> {
    let space = space_arg(fn_name, space)?;
    if ENCODING_SPACES.contains(&space) {
        Ok(space)
    } else {
        Err(bad_arg(
            fn_name,
            format!(
                "color space '{space}' has no transfer function; use one of: {}",
                ENCODING_SPACES.join(", ")
            ),
        ))
    }
}

/// 空间使用的传输函数名。
///
/// The transfer function a space uses.
pub fn transfer_function(fn_name: &str, space: &str) -> Result<&'static str, ExtensionError> {
    let space = encoding_space(fn_name, space)?;
    Ok(match space {
        "srgb" | "display_p3" | "luma" => "srgb",
        "srgb_gamma" | "luma_gamma" => "gamma_2_2",
        "adobe_rgb" => "adobe_rgb",
        "prophoto_rgb" => "prophoto_rgb",
        "dci_p3" => "p3_gamma",
        "rec709" | "rec2020" => "rec_oetf",
        _ => "linear",
    })
}

/// 编码值 → 线性值（palette 的 `IntoLinear`）。
///
/// Decode one encoded value into linear light.
pub fn decode(fn_name: &str, space: &str, value: f64) -> Result<f64, ExtensionError> {
    Ok(match encoding_space(fn_name, space)? {
        "srgb" | "display_p3" | "luma" => SrgbEncoding::into_linear(value),
        "srgb_gamma" | "luma_gamma" => GammaFn::<F2p2>::into_linear(value),
        "adobe_rgb" => AdobeRgbEncoding::into_linear(value),
        "prophoto_rgb" => ProPhotoRgbEncoding::into_linear(value),
        "dci_p3" => P3Gamma::into_linear(value),
        "rec709" | "rec2020" => RecOetf::into_linear(value),
        _ => LinearFn::into_linear(value),
    })
}

/// 线性值 → 编码值（palette 的 `FromLinear`）。
///
/// Encode one linear value.
pub fn encode(fn_name: &str, space: &str, value: f64) -> Result<f64, ExtensionError> {
    Ok(match encoding_space(fn_name, space)? {
        "srgb" | "display_p3" | "luma" => SrgbEncoding::from_linear(value),
        "srgb_gamma" | "luma_gamma" => GammaFn::<F2p2>::from_linear(value),
        "adobe_rgb" => AdobeRgbEncoding::from_linear(value),
        "prophoto_rgb" => ProPhotoRgbEncoding::from_linear(value),
        "dci_p3" => P3Gamma::from_linear(value),
        "rec709" | "rec2020" => RecOetf::from_linear(value),
        _ => LinearFn::from_linear(value),
    })
}

/// 空间的参考白点名。
///
/// The white point name of a space.
pub fn white_point_of(fn_name: &str, space: &str) -> Result<&'static str, ExtensionError> {
    Ok(match space_arg(fn_name, space)? {
        "xyz_d50" | "prophoto_rgb" | "prophoto_rgb_linear" => "d50",
        "dci_p3" | "dci_p3_linear" => "dci_p3",
        _ => "d65",
    })
}

/// 一组原色的类型标签：把「用哪个 `RgbSpace` 取原色/矩阵」收进一个宏，避免 15 份重复代码。
///
/// Resolve an RGB space name into its palette `RgbSpace` type for a small expression.
macro_rules! with_rgb_space {
    ($space:expr, $fn_name:expr, $what:literal, $ty:ident => $body:expr,
     [$($name:pat => $real:ty),+ $(,)?]) => {{
        match $space {
            $(
                $name => {
                    type $ty = $real;
                    $body
                }
            )+
            other => {
                return Err(bad_arg(
                    $fn_name,
                    format!(
                        "color space '{}' has no {}; use an RGB space: srgb, adobe_rgb, \
                         display_p3, dci_p3, prophoto_rgb, rec709 or rec2020",
                        other, $what
                    ),
                ));
            }
        }
    }};
}

/// 一个 RGB 空间的三原色（红、绿、蓝），以 yxy 颜色返回。
///
/// The three primaries of an RGB space as yxy colors.
fn primaries_of<S: RgbSpace>() -> [(f64, f64, f64); 3]
where
    S::Primaries: Primaries<f64>,
{
    let (r, g, b) = (
        S::Primaries::red(),
        S::Primaries::green(),
        S::Primaries::blue(),
    );
    [
        r.into_components(),
        g.into_components(),
        b.into_components(),
    ]
}

/// RGB 空间的原色列表。
///
/// The primaries of an RGB space, as yxy colors.
pub fn primaries(fn_name: &str, space: &str) -> Result<Vec<PltColor>, ExtensionError> {
    let space = space_arg(fn_name, space)?;
    let triples = with_rgb_space!(space, fn_name, "RGB primaries", S => primaries_of::<S>(), [
        "srgb" | "srgb_linear" | "srgb_gamma" | "rec709" | "rec709_linear"
        | "luma" | "luma_linear" | "luma_gamma" => Srgb,
        "adobe_rgb" | "adobe_rgb_linear" => AdobeRgb,
        "display_p3" | "display_p3_linear" => DisplayP3Space,
        "dci_p3" | "dci_p3_linear" => DciP3Space,
        "prophoto_rgb" | "prophoto_rgb_linear" => ProPhotoRgb,
        "rec2020" | "rec2020_linear" => Rec2020,
    ]);
    Ok(triples
        .into_iter()
        .map(|(x, y, luma)| PltColor::new("yxy", x, y, luma, 1.0))
        .collect())
}

/// RGB → XYZ 的 3x3 矩阵（行主序 9 个数）。
///
/// The RGB→XYZ matrix, row-major.
pub fn rgb_to_xyz(fn_name: &str, space: &str) -> Result<[f64; 9], ExtensionError> {
    let space = space_arg(fn_name, space)?;
    Ok(
        with_rgb_space!(space, fn_name, "RGB→XYZ matrix", S => rgb_to_xyz_matrix::<S, f64>(), [
            "srgb" | "srgb_linear" | "srgb_gamma" | "rec709" | "rec709_linear"
            | "luma" | "luma_linear" | "luma_gamma" => Srgb,
            "adobe_rgb" | "adobe_rgb_linear" => AdobeRgb,
            "display_p3" | "display_p3_linear" => DisplayP3Space,
            "dci_p3" | "dci_p3_linear" => DciP3Space,
            "prophoto_rgb" | "prophoto_rgb_linear" => ProPhotoRgb,
            "rec2020" | "rec2020_linear" => Rec2020,
        ]),
    )
}

/// XYZ → RGB 的 3x3 矩阵（行主序 9 个数），即上面那个矩阵的逆。
///
/// The XYZ→RGB matrix: the inverse of the RGB→XYZ matrix.
pub fn xyz_to_rgb(fn_name: &str, space: &str) -> Result<[f64; 9], ExtensionError> {
    Ok(matrix_inverse(rgb_to_xyz(fn_name, space)?))
}
