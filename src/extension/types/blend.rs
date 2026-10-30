// ============================================================================
// 混合（`Blend`，11 种可分離模式）与合成（`Compose`，6 种 Porter-Duff 操作），
// 以及 OpenGL 风格的自定义因子（`BlendWith` + `Equations`）。
//
// palette 把这三组能力放在 `blend` 模块里，都按「预乘 alpha」定义、作用于 `Alpha<C, T>`：
//   - `Blend` 的 11 个模式额外要求 `StimulusColor`，因此只在 RGB（全部编码）、Luma、Lms、
//     Xyz 上有定义；
//   - `Compose` 与 `BlendWith` 只要求 `Premultiply`，因此在上述之外还覆盖 Lab、Luv、Oklab、
//     Yxy、Cam16UcsJab。
// 其余空间（Hsl/Lch/… 的 hue 没有逐分量混合的意义）会得到查询错误，提示先 `plt_convert`。
//
// 参数顺序与 palette 一致：`a.over(b)` / `a.multiply(b)` 里的 `a` 是源（source），`b` 是背景
// （destination）。第二个颜色若不在第一个颜色的空间里，会先转过去。
//
// Blending (11 separable modes), composition (6 Porter-Duff operators) and OpenGL-style
// `Equations`, all premultiplied-alpha operations on `Alpha<C, T>`. `Blend` additionally needs
// `StimulusColor`, so it exists for RGB, Luma, Lms and Xyz only; `Compose` and `BlendWith` need
// just `Premultiply` and also cover Lab, Luv, Oklab, Yxy and Cam16UcsJab. Argument order matches
// palette: `a` is the source, `b` the destination.
// ============================================================================

// 同 color.rs：`srgb_gamma` / `luma_gamma` 用的是 palette 标为 deprecated 的 gamma 2.2 类型。
#![allow(deprecated)]

use palette::Alpha;
use palette::blend::{Blend, BlendWith, Compose, Equations, Parameter};
use palette::cam16::Cam16UcsJab;
use palette::encoding;
use palette::lms::{BradfordLms, VonKriesLms};
use palette::luma::Luma;
use palette::rgb::{
    AdobeRgb, DciP3, DisplayP3, GammaSrgb, LinAdobeRgb, LinDciP3, LinDisplayP3, LinProPhotoRgb,
    LinRec2020, LinSrgb, ProPhotoRgb, Rec709, Rec2020, Srgb,
};
use palette::white_point::{D50, D65};
use palette::{Lab, Luv, Oklab, Xyz, Yxy};
use quack_rs::error::ExtensionError;

use super::color::{PltColor, bad_arg};
use super::ops::SqlComponents;

/// 单个颜色的混合结果：三分量 + alpha。
///
/// A single blend result: three components plus alpha.
type AlphaOut = ((f64, f64, f64), f64);

/// 把两个颜色各自包成 `Alpha<C, T>` 交给 `f`，再把结果摊开。
///
/// Wrap both colors as `Alpha<C, T>` for `f`, then flatten the result.
fn via_alpha<C, F>(a: C, alpha_a: f64, b: C, alpha_b: f64, f: F) -> AlphaOut
where
    C: SqlComponents,
    F: FnOnce(Alpha<C, f64>, Alpha<C, f64>) -> Alpha<C, f64>,
{
    let out = f(
        Alpha {
            color: a,
            alpha: alpha_a,
        },
        Alpha {
            color: b,
            alpha: alpha_b,
        },
    );
    (out.color.to_triple(), out.alpha)
}

/// 空间不支持 `Blend` 模式时的统一错误。
///
/// Uniform "space cannot blend" error.
fn unsupported_blend(fn_name: &str, space: &str) -> ExtensionError {
    bad_arg(
        fn_name,
        format!(
            "color space '{space}' does not support the blend modes (they need an RGB-like \
             stimulus space); convert it first, e.g. plt_convert(color, 'srgb_linear')"
        ),
    )
}

/// 空间不支持 `Compose` / `BlendWith` 时的统一错误。
///
/// Uniform "space cannot compose" error.
fn unsupported_compose(fn_name: &str, space: &str) -> ExtensionError {
    bad_arg(
        fn_name,
        format!(
            "color space '{space}' does not support composition or custom blend equations; \
             convert it first, e.g. plt_convert(color, 'srgb_linear')"
        ),
    )
}

/// 一个 `Blend` 模式在全部可混合空间（RGB / Luma / Lms / Xyz）上的分派。
///
/// One `Blend` mode dispatched over every blendable stimulus space.
macro_rules! blend_op {
    ($space:expr, $fn_name:expr, $a:expr, $aa:expr, $b:expr, $ab:expr, $method:ident) => {{
        let (a, aa, b, ab) = ($a, $aa, $b, $ab);
        match $space {
            "srgb" => Ok(via_alpha(
                Srgb::<f64>::from_triple(a),
                aa,
                Srgb::<f64>::from_triple(b),
                ab,
                |x, y| x.$method(y),
            )),
            "srgb_linear" => Ok(via_alpha(
                LinSrgb::<f64>::from_triple(a),
                aa,
                LinSrgb::<f64>::from_triple(b),
                ab,
                |x, y| x.$method(y),
            )),
            "srgb_gamma" => Ok(via_alpha(
                GammaSrgb::<f64>::from_triple(a),
                aa,
                GammaSrgb::<f64>::from_triple(b),
                ab,
                |x, y| x.$method(y),
            )),
            "adobe_rgb" => Ok(via_alpha(
                AdobeRgb::<f64>::from_triple(a),
                aa,
                AdobeRgb::<f64>::from_triple(b),
                ab,
                |x, y| x.$method(y),
            )),
            "adobe_rgb_linear" => Ok(via_alpha(
                LinAdobeRgb::<f64>::from_triple(a),
                aa,
                LinAdobeRgb::<f64>::from_triple(b),
                ab,
                |x, y| x.$method(y),
            )),
            "display_p3" => Ok(via_alpha(
                DisplayP3::<f64>::from_triple(a),
                aa,
                DisplayP3::<f64>::from_triple(b),
                ab,
                |x, y| x.$method(y),
            )),
            "display_p3_linear" => Ok(via_alpha(
                LinDisplayP3::<f64>::from_triple(a),
                aa,
                LinDisplayP3::<f64>::from_triple(b),
                ab,
                |x, y| x.$method(y),
            )),
            "dci_p3" => Ok(via_alpha(
                DciP3::<f64>::from_triple(a),
                aa,
                DciP3::<f64>::from_triple(b),
                ab,
                |x, y| x.$method(y),
            )),
            "dci_p3_linear" => Ok(via_alpha(
                LinDciP3::<f64>::from_triple(a),
                aa,
                LinDciP3::<f64>::from_triple(b),
                ab,
                |x, y| x.$method(y),
            )),
            "prophoto_rgb" => Ok(via_alpha(
                ProPhotoRgb::<f64>::from_triple(a),
                aa,
                ProPhotoRgb::<f64>::from_triple(b),
                ab,
                |x, y| x.$method(y),
            )),
            "prophoto_rgb_linear" => Ok(via_alpha(
                LinProPhotoRgb::<f64>::from_triple(a),
                aa,
                LinProPhotoRgb::<f64>::from_triple(b),
                ab,
                |x, y| x.$method(y),
            )),
            "rec709" => Ok(via_alpha(
                Rec709::<f64>::from_triple(a),
                aa,
                Rec709::<f64>::from_triple(b),
                ab,
                |x, y| x.$method(y),
            )),
            "rec709_linear" => Ok(via_alpha(
                LinSrgb::<f64>::from_triple(a),
                aa,
                LinSrgb::<f64>::from_triple(b),
                ab,
                |x, y| x.$method(y),
            )),
            "rec2020" => Ok(via_alpha(
                Rec2020::<f64>::from_triple(a),
                aa,
                Rec2020::<f64>::from_triple(b),
                ab,
                |x, y| x.$method(y),
            )),
            "rec2020_linear" => Ok(via_alpha(
                LinRec2020::<f64>::from_triple(a),
                aa,
                LinRec2020::<f64>::from_triple(b),
                ab,
                |x, y| x.$method(y),
            )),
            "xyz" => Ok(via_alpha(
                Xyz::<D65, f64>::from_triple(a),
                aa,
                Xyz::<D65, f64>::from_triple(b),
                ab,
                |x, y| x.$method(y),
            )),
            "xyz_d50" => Ok(via_alpha(
                Xyz::<D50, f64>::from_triple(a),
                aa,
                Xyz::<D50, f64>::from_triple(b),
                ab,
                |x, y| x.$method(y),
            )),
            "luma" => Ok(via_alpha(
                Luma::<encoding::Srgb, f64>::from_triple(a),
                aa,
                Luma::<encoding::Srgb, f64>::from_triple(b),
                ab,
                |x, y| x.$method(y),
            )),
            "luma_linear" => Ok(via_alpha(
                Luma::<encoding::Linear<D65>, f64>::from_triple(a),
                aa,
                Luma::<encoding::Linear<D65>, f64>::from_triple(b),
                ab,
                |x, y| x.$method(y),
            )),
            "luma_gamma" => Ok(via_alpha(
                Luma::<encoding::Gamma<D65>, f64>::from_triple(a),
                aa,
                Luma::<encoding::Gamma<D65>, f64>::from_triple(b),
                ab,
                |x, y| x.$method(y),
            )),
            "lms" => Ok(via_alpha(
                BradfordLms::<D65, f64>::from_triple(a),
                aa,
                BradfordLms::<D65, f64>::from_triple(b),
                ab,
                |x, y| x.$method(y),
            )),
            "lms_von_kries" => Ok(via_alpha(
                VonKriesLms::<D65, f64>::from_triple(a),
                aa,
                VonKriesLms::<D65, f64>::from_triple(b),
                ab,
                |x, y| x.$method(y),
            )),
            other => Err(unsupported_blend($fn_name, other)),
        }
    }};
}

/// 一个 `Compose` 操作在全部可预乘空间上的分派。
///
/// One `Compose` operator dispatched over every premultipliable space.
macro_rules! compose_op {
    ($space:expr, $fn_name:expr, $a:expr, $aa:expr, $b:expr, $ab:expr, $method:ident) => {{
        let (a, aa, b, ab) = ($a, $aa, $b, $ab);
        match $space {
            "srgb" => Ok(via_alpha(
                Srgb::<f64>::from_triple(a),
                aa,
                Srgb::<f64>::from_triple(b),
                ab,
                |x, y| x.$method(y),
            )),
            "srgb_linear" => Ok(via_alpha(
                LinSrgb::<f64>::from_triple(a),
                aa,
                LinSrgb::<f64>::from_triple(b),
                ab,
                |x, y| x.$method(y),
            )),
            "srgb_gamma" => Ok(via_alpha(
                GammaSrgb::<f64>::from_triple(a),
                aa,
                GammaSrgb::<f64>::from_triple(b),
                ab,
                |x, y| x.$method(y),
            )),
            "adobe_rgb" => Ok(via_alpha(
                AdobeRgb::<f64>::from_triple(a),
                aa,
                AdobeRgb::<f64>::from_triple(b),
                ab,
                |x, y| x.$method(y),
            )),
            "adobe_rgb_linear" => Ok(via_alpha(
                LinAdobeRgb::<f64>::from_triple(a),
                aa,
                LinAdobeRgb::<f64>::from_triple(b),
                ab,
                |x, y| x.$method(y),
            )),
            "display_p3" => Ok(via_alpha(
                DisplayP3::<f64>::from_triple(a),
                aa,
                DisplayP3::<f64>::from_triple(b),
                ab,
                |x, y| x.$method(y),
            )),
            "display_p3_linear" => Ok(via_alpha(
                LinDisplayP3::<f64>::from_triple(a),
                aa,
                LinDisplayP3::<f64>::from_triple(b),
                ab,
                |x, y| x.$method(y),
            )),
            "dci_p3" => Ok(via_alpha(
                DciP3::<f64>::from_triple(a),
                aa,
                DciP3::<f64>::from_triple(b),
                ab,
                |x, y| x.$method(y),
            )),
            "dci_p3_linear" => Ok(via_alpha(
                LinDciP3::<f64>::from_triple(a),
                aa,
                LinDciP3::<f64>::from_triple(b),
                ab,
                |x, y| x.$method(y),
            )),
            "prophoto_rgb" => Ok(via_alpha(
                ProPhotoRgb::<f64>::from_triple(a),
                aa,
                ProPhotoRgb::<f64>::from_triple(b),
                ab,
                |x, y| x.$method(y),
            )),
            "prophoto_rgb_linear" => Ok(via_alpha(
                LinProPhotoRgb::<f64>::from_triple(a),
                aa,
                LinProPhotoRgb::<f64>::from_triple(b),
                ab,
                |x, y| x.$method(y),
            )),
            "rec709" => Ok(via_alpha(
                Rec709::<f64>::from_triple(a),
                aa,
                Rec709::<f64>::from_triple(b),
                ab,
                |x, y| x.$method(y),
            )),
            "rec709_linear" => Ok(via_alpha(
                LinSrgb::<f64>::from_triple(a),
                aa,
                LinSrgb::<f64>::from_triple(b),
                ab,
                |x, y| x.$method(y),
            )),
            "rec2020" => Ok(via_alpha(
                Rec2020::<f64>::from_triple(a),
                aa,
                Rec2020::<f64>::from_triple(b),
                ab,
                |x, y| x.$method(y),
            )),
            "rec2020_linear" => Ok(via_alpha(
                LinRec2020::<f64>::from_triple(a),
                aa,
                LinRec2020::<f64>::from_triple(b),
                ab,
                |x, y| x.$method(y),
            )),
            "lab" => Ok(via_alpha(
                Lab::<D65, f64>::from_triple(a),
                aa,
                Lab::<D65, f64>::from_triple(b),
                ab,
                |x, y| x.$method(y),
            )),
            "luv" => Ok(via_alpha(
                Luv::<D65, f64>::from_triple(a),
                aa,
                Luv::<D65, f64>::from_triple(b),
                ab,
                |x, y| x.$method(y),
            )),
            "xyz" => Ok(via_alpha(
                Xyz::<D65, f64>::from_triple(a),
                aa,
                Xyz::<D65, f64>::from_triple(b),
                ab,
                |x, y| x.$method(y),
            )),
            "xyz_d50" => Ok(via_alpha(
                Xyz::<D50, f64>::from_triple(a),
                aa,
                Xyz::<D50, f64>::from_triple(b),
                ab,
                |x, y| x.$method(y),
            )),
            "yxy" => Ok(via_alpha(
                Yxy::<D65, f64>::from_triple(a),
                aa,
                Yxy::<D65, f64>::from_triple(b),
                ab,
                |x, y| x.$method(y),
            )),
            "luma" => Ok(via_alpha(
                Luma::<encoding::Srgb, f64>::from_triple(a),
                aa,
                Luma::<encoding::Srgb, f64>::from_triple(b),
                ab,
                |x, y| x.$method(y),
            )),
            "luma_linear" => Ok(via_alpha(
                Luma::<encoding::Linear<D65>, f64>::from_triple(a),
                aa,
                Luma::<encoding::Linear<D65>, f64>::from_triple(b),
                ab,
                |x, y| x.$method(y),
            )),
            "luma_gamma" => Ok(via_alpha(
                Luma::<encoding::Gamma<D65>, f64>::from_triple(a),
                aa,
                Luma::<encoding::Gamma<D65>, f64>::from_triple(b),
                ab,
                |x, y| x.$method(y),
            )),
            "oklab" => Ok(via_alpha(
                Oklab::<f64>::from_triple(a),
                aa,
                Oklab::<f64>::from_triple(b),
                ab,
                |x, y| x.$method(y),
            )),
            "cam16_ucs_jab" => Ok(via_alpha(
                Cam16UcsJab::<f64>::from_triple(a),
                aa,
                Cam16UcsJab::<f64>::from_triple(b),
                ab,
                |x, y| x.$method(y),
            )),
            "lms" => Ok(via_alpha(
                BradfordLms::<D65, f64>::from_triple(a),
                aa,
                BradfordLms::<D65, f64>::from_triple(b),
                ab,
                |x, y| x.$method(y),
            )),
            "lms_von_kries" => Ok(via_alpha(
                VonKriesLms::<D65, f64>::from_triple(a),
                aa,
                VonKriesLms::<D65, f64>::from_triple(b),
                ab,
                |x, y| x.$method(y),
            )),
            other => Err(unsupported_compose($fn_name, other)),
        }
    }};
}

/// OpenGL 风格 `Equations` 的分派（与 `Compose` 同一批空间）。
///
/// Dispatch for the OpenGL-style `Equations` (same spaces as `Compose`).
macro_rules! equations_op {
    ($space:expr, $fn_name:expr, $a:expr, $aa:expr, $b:expr, $ab:expr, $source:expr, $dest:expr) => {{
        let (a, aa, b, ab) = ($a, $aa, $b, $ab);
        let equations = Equations::from_parameters($source, $dest);
        match $space {
            "srgb" => Ok(via_alpha(
                Srgb::<f64>::from_triple(a),
                aa,
                Srgb::<f64>::from_triple(b),
                ab,
                |x, y| x.blend_with(y, equations),
            )),
            "srgb_linear" => Ok(via_alpha(
                LinSrgb::<f64>::from_triple(a),
                aa,
                LinSrgb::<f64>::from_triple(b),
                ab,
                |x, y| x.blend_with(y, equations),
            )),
            "srgb_gamma" => Ok(via_alpha(
                GammaSrgb::<f64>::from_triple(a),
                aa,
                GammaSrgb::<f64>::from_triple(b),
                ab,
                |x, y| x.blend_with(y, equations),
            )),
            "adobe_rgb" => Ok(via_alpha(
                AdobeRgb::<f64>::from_triple(a),
                aa,
                AdobeRgb::<f64>::from_triple(b),
                ab,
                |x, y| x.blend_with(y, equations),
            )),
            "adobe_rgb_linear" => Ok(via_alpha(
                LinAdobeRgb::<f64>::from_triple(a),
                aa,
                LinAdobeRgb::<f64>::from_triple(b),
                ab,
                |x, y| x.blend_with(y, equations),
            )),
            "display_p3" => Ok(via_alpha(
                DisplayP3::<f64>::from_triple(a),
                aa,
                DisplayP3::<f64>::from_triple(b),
                ab,
                |x, y| x.blend_with(y, equations),
            )),
            "display_p3_linear" => Ok(via_alpha(
                LinDisplayP3::<f64>::from_triple(a),
                aa,
                LinDisplayP3::<f64>::from_triple(b),
                ab,
                |x, y| x.blend_with(y, equations),
            )),
            "dci_p3" => Ok(via_alpha(
                DciP3::<f64>::from_triple(a),
                aa,
                DciP3::<f64>::from_triple(b),
                ab,
                |x, y| x.blend_with(y, equations),
            )),
            "dci_p3_linear" => Ok(via_alpha(
                LinDciP3::<f64>::from_triple(a),
                aa,
                LinDciP3::<f64>::from_triple(b),
                ab,
                |x, y| x.blend_with(y, equations),
            )),
            "prophoto_rgb" => Ok(via_alpha(
                ProPhotoRgb::<f64>::from_triple(a),
                aa,
                ProPhotoRgb::<f64>::from_triple(b),
                ab,
                |x, y| x.blend_with(y, equations),
            )),
            "prophoto_rgb_linear" => Ok(via_alpha(
                LinProPhotoRgb::<f64>::from_triple(a),
                aa,
                LinProPhotoRgb::<f64>::from_triple(b),
                ab,
                |x, y| x.blend_with(y, equations),
            )),
            "rec709" => Ok(via_alpha(
                Rec709::<f64>::from_triple(a),
                aa,
                Rec709::<f64>::from_triple(b),
                ab,
                |x, y| x.blend_with(y, equations),
            )),
            "rec709_linear" => Ok(via_alpha(
                LinSrgb::<f64>::from_triple(a),
                aa,
                LinSrgb::<f64>::from_triple(b),
                ab,
                |x, y| x.blend_with(y, equations),
            )),
            "rec2020" => Ok(via_alpha(
                Rec2020::<f64>::from_triple(a),
                aa,
                Rec2020::<f64>::from_triple(b),
                ab,
                |x, y| x.blend_with(y, equations),
            )),
            "rec2020_linear" => Ok(via_alpha(
                LinRec2020::<f64>::from_triple(a),
                aa,
                LinRec2020::<f64>::from_triple(b),
                ab,
                |x, y| x.blend_with(y, equations),
            )),
            "lab" => Ok(via_alpha(
                Lab::<D65, f64>::from_triple(a),
                aa,
                Lab::<D65, f64>::from_triple(b),
                ab,
                |x, y| x.blend_with(y, equations),
            )),
            "luv" => Ok(via_alpha(
                Luv::<D65, f64>::from_triple(a),
                aa,
                Luv::<D65, f64>::from_triple(b),
                ab,
                |x, y| x.blend_with(y, equations),
            )),
            "xyz" => Ok(via_alpha(
                Xyz::<D65, f64>::from_triple(a),
                aa,
                Xyz::<D65, f64>::from_triple(b),
                ab,
                |x, y| x.blend_with(y, equations),
            )),
            "xyz_d50" => Ok(via_alpha(
                Xyz::<D50, f64>::from_triple(a),
                aa,
                Xyz::<D50, f64>::from_triple(b),
                ab,
                |x, y| x.blend_with(y, equations),
            )),
            "yxy" => Ok(via_alpha(
                Yxy::<D65, f64>::from_triple(a),
                aa,
                Yxy::<D65, f64>::from_triple(b),
                ab,
                |x, y| x.blend_with(y, equations),
            )),
            "luma" => Ok(via_alpha(
                Luma::<encoding::Srgb, f64>::from_triple(a),
                aa,
                Luma::<encoding::Srgb, f64>::from_triple(b),
                ab,
                |x, y| x.blend_with(y, equations),
            )),
            "luma_linear" => Ok(via_alpha(
                Luma::<encoding::Linear<D65>, f64>::from_triple(a),
                aa,
                Luma::<encoding::Linear<D65>, f64>::from_triple(b),
                ab,
                |x, y| x.blend_with(y, equations),
            )),
            "luma_gamma" => Ok(via_alpha(
                Luma::<encoding::Gamma<D65>, f64>::from_triple(a),
                aa,
                Luma::<encoding::Gamma<D65>, f64>::from_triple(b),
                ab,
                |x, y| x.blend_with(y, equations),
            )),
            "oklab" => Ok(via_alpha(
                Oklab::<f64>::from_triple(a),
                aa,
                Oklab::<f64>::from_triple(b),
                ab,
                |x, y| x.blend_with(y, equations),
            )),
            "cam16_ucs_jab" => Ok(via_alpha(
                Cam16UcsJab::<f64>::from_triple(a),
                aa,
                Cam16UcsJab::<f64>::from_triple(b),
                ab,
                |x, y| x.blend_with(y, equations),
            )),
            "lms" => Ok(via_alpha(
                BradfordLms::<D65, f64>::from_triple(a),
                aa,
                BradfordLms::<D65, f64>::from_triple(b),
                ab,
                |x, y| x.blend_with(y, equations),
            )),
            "lms_von_kries" => Ok(via_alpha(
                VonKriesLms::<D65, f64>::from_triple(a),
                aa,
                VonKriesLms::<D65, f64>::from_triple(b),
                ab,
                |x, y| x.blend_with(y, equations),
            )),
            other => Err(unsupported_compose($fn_name, other)),
        }
    }};
}

/// 混合模式名（规范化后）。
///
/// Blend mode names, normalized.
pub const BLEND_MODES: &[&str] = &[
    "multiply",
    "screen",
    "overlay",
    "darken",
    "lighten",
    "dodge",
    "burn",
    "hard_light",
    "soft_light",
    "difference",
    "exclusion",
];

/// 合成模式名（规范化后）。
///
/// Composition operator names, normalized.
pub const COMPOSE_MODES: &[&str] = &["over", "inside", "outside", "atop", "xor", "plus"];

/// `Equations` 的因子名（规范化后）。
///
/// Factor names for `Equations`, normalized.
pub const BLEND_PARAMETERS: &[&str] = &[
    "zero",
    "one",
    "source_color",
    "one_minus_source_color",
    "destination_color",
    "one_minus_destination_color",
    "source_alpha",
    "one_minus_source_alpha",
    "destination_alpha",
    "one_minus_destination_alpha",
];

/// 把模式名/因子名规范化：小写，`-` 与空格视作 `_`。
///
/// Normalize a mode or factor name: lowercase, `-` and spaces become `_`.
fn normalize(name: &str) -> String {
    name.trim().to_ascii_lowercase().replace(['-', ' '], "_")
}

/// 解析 `Blend` 模式名。
///
/// Resolve a `Blend` mode name.
fn blend_mode(fn_name: &str, mode: &str) -> Result<&'static str, ExtensionError> {
    let name = normalize(mode);
    BLEND_MODES
        .iter()
        .copied()
        .find(|m| *m == name)
        .ok_or_else(|| {
            bad_arg(
                fn_name,
                format!(
                    "unknown blend mode '{mode}'; supported: {}",
                    BLEND_MODES.join(", ")
                ),
            )
        })
}

/// 解析 `Compose` 操作名。
///
/// Resolve a `Compose` operator name.
fn compose_mode(fn_name: &str, mode: &str) -> Result<&'static str, ExtensionError> {
    let name = normalize(mode);
    COMPOSE_MODES
        .iter()
        .copied()
        .find(|m| *m == name)
        .ok_or_else(|| {
            bad_arg(
                fn_name,
                format!(
                    "unknown compose mode '{mode}'; supported: {}",
                    COMPOSE_MODES.join(", ")
                ),
            )
        })
}

/// 解析 `Equations` 的因子名。
///
/// Resolve an `Equations` factor name.
pub fn blend_parameter(fn_name: &str, name: &str) -> Result<Parameter, ExtensionError> {
    let normalized = normalize(name);
    let parameter = match normalized.as_str() {
        "zero" => Parameter::Zero,
        "one" => Parameter::One,
        "source_color" => Parameter::SourceColor,
        "one_minus_source_color" => Parameter::OneMinusSourceColor,
        "destination_color" => Parameter::DestinationColor,
        "one_minus_destination_color" => Parameter::OneMinusDestinationColor,
        "source_alpha" => Parameter::SourceAlpha,
        "one_minus_source_alpha" => Parameter::OneMinusSourceAlpha,
        "destination_alpha" => Parameter::DestinationAlpha,
        "one_minus_destination_alpha" => Parameter::OneMinusDestinationAlpha,
        other => {
            return Err(bad_arg(
                fn_name,
                format!(
                    "unknown blend factor '{other}'; supported: {}",
                    BLEND_PARAMETERS.join(", ")
                ),
            ));
        }
    };
    Ok(parameter)
}

impl PltColor {
    /// `Blend`：`self`（源）叠加到 `other`（背景）上。
    ///
    /// `Blend`: `self` (source) blended onto `other` (destination).
    pub fn blend(&self, other: &Self, mode: &str, fn_name: &str) -> Result<Self, ExtensionError> {
        let mode = blend_mode(fn_name, mode)?;
        let (space, other) = self.same_space(other, fn_name)?;
        let (a, aa) = ((self.c1, self.c2, self.c3), self.alpha);
        let (b, ab) = ((other.c1, other.c2, other.c3), other.alpha);
        let (t, alpha) = match mode {
            "multiply" => blend_op!(space, fn_name, a, aa, b, ab, multiply),
            "screen" => blend_op!(space, fn_name, a, aa, b, ab, screen),
            "overlay" => blend_op!(space, fn_name, a, aa, b, ab, overlay),
            "darken" => blend_op!(space, fn_name, a, aa, b, ab, darken),
            "lighten" => blend_op!(space, fn_name, a, aa, b, ab, lighten),
            "dodge" => blend_op!(space, fn_name, a, aa, b, ab, dodge),
            "burn" => blend_op!(space, fn_name, a, aa, b, ab, burn),
            "hard_light" => blend_op!(space, fn_name, a, aa, b, ab, hard_light),
            "soft_light" => blend_op!(space, fn_name, a, aa, b, ab, soft_light),
            "difference" => blend_op!(space, fn_name, a, aa, b, ab, difference),
            "exclusion" => blend_op!(space, fn_name, a, aa, b, ab, exclusion),
            _ => unreachable!("blend_mode returned an unknown mode"),
        }?;
        Ok(Self::new(space, t.0, t.1, t.2, alpha))
    }

    /// `Compose`：`self`（源）与 `other`（背景）按 Porter-Duff 规则合成。
    ///
    /// `Compose`: `self` (source) composited with `other` (destination).
    pub fn compose(&self, other: &Self, mode: &str, fn_name: &str) -> Result<Self, ExtensionError> {
        let mode = compose_mode(fn_name, mode)?;
        let (space, other) = self.same_space(other, fn_name)?;
        let (a, aa) = ((self.c1, self.c2, self.c3), self.alpha);
        let (b, ab) = ((other.c1, other.c2, other.c3), other.alpha);
        let (t, alpha) = match mode {
            "over" => compose_op!(space, fn_name, a, aa, b, ab, over),
            "inside" => compose_op!(space, fn_name, a, aa, b, ab, inside),
            "outside" => compose_op!(space, fn_name, a, aa, b, ab, outside),
            "atop" => compose_op!(space, fn_name, a, aa, b, ab, atop),
            "xor" => compose_op!(space, fn_name, a, aa, b, ab, xor),
            "plus" => compose_op!(space, fn_name, a, aa, b, ab, plus),
            _ => unreachable!("compose_mode returned an unknown mode"),
        }?;
        Ok(Self::new(space, t.0, t.1, t.2, alpha))
    }

    /// `BlendWith` + `Equations`：用源/目标因子做 OpenGL 式混合。
    ///
    /// `BlendWith` with `Equations`: OpenGL-style blending from source/destination factors.
    pub fn blend_equations(
        &self,
        other: &Self,
        source: &str,
        destination: &str,
        fn_name: &str,
    ) -> Result<Self, ExtensionError> {
        let source = blend_parameter(fn_name, source)?;
        let destination = blend_parameter(fn_name, destination)?;
        let (space, other) = self.same_space(other, fn_name)?;
        let (a, aa) = ((self.c1, self.c2, self.c3), self.alpha);
        let (b, ab) = ((other.c1, other.c2, other.c3), other.alpha);
        let (t, alpha) = equations_op!(space, fn_name, a, aa, b, ab, source, destination)?;
        Ok(Self::new(space, t.0, t.1, t.2, alpha))
    }

    /// 把 `other` 转到 `self` 的空间（同空间则原样）。
    ///
    /// Convert `other` into `self`'s space, or keep it as is when it already is there.
    fn same_space<'a>(
        &self,
        other: &'a Self,
        fn_name: &str,
    ) -> Result<(&'static str, std::borrow::Cow<'a, Self>), ExtensionError> {
        let space = self.require_space(fn_name)?;
        if other.require_space(fn_name)? == space {
            Ok((space, std::borrow::Cow::Borrowed(other)))
        } else {
            Ok((
                space,
                std::borrow::Cow::Owned(other.convert(fn_name, space)?),
            ))
        }
    }
}
