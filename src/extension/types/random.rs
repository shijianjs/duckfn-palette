// ============================================================================
// 随机色（palette 的 `random_sampling`，即各颜色类型对 rand 的 `Standard` 分布实现）。
//
// palette 为每个空间定义了「在该空间内均匀」的采样范围：RGB 是 0..1，Lab 的 a/b 覆盖
// -128..127，Lch 的色度到该空间的最大值，Hsv/Hsl/Hwb 按锥体/双锥体体积加权，Xyz 按白点
// 缩放，CAM16-UCS 按 sRGB 能到达的边界……这些映射都在 palette 里，这里只负责把采样结果
// 摊成 `plt_color`。
//
// 只覆盖 palette 实现了 `Standard` 的空间；`cam16_jch` 之类的部分类型、`hsluv` 与
// `okhsl`/`okhsv`/`okhwb` 没有随机分布（palette 没给），调用会得到带函数名的错误。
// 种子让结果可复现：同一空间 + 同一种子恒定得到同一个颜色。
//
// Random colors via palette's `Standard` distributions. palette defines what "uniform inside the
// space" means (RGB 0..1, Lab a/b over -128..127, Lch up to max chroma, Hsv/Hsl/Hwb weighted by
// the solid's volume, ...); this module only flattens the result into `plt_color`. Spaces without
// a palette distribution (partial CAM16, Hsluv, Okhsl/Okhsv/Okhwb) produce an error. Seeding keeps
// results reproducible.
// ============================================================================

// 随机色覆盖 `srgb_gamma` / `luma_gamma`，palette 把 gamma 2.2 编码标记为 deprecated，
// 但它仍是公开能力（与 color.rs 的取舍相同）。
// The gamma 2.2 spaces are deprecated but still public; sample them like any other space.
#![allow(deprecated)]

use palette::cam16::{Cam16UcsJab, Cam16UcsJmh};
use palette::encoding::{Gamma, Linear, Srgb as SrgbEncoding};
use palette::lms::{BradfordLms, VonKriesLms};
use palette::luma::Luma;
use palette::rgb::{
    AdobeRgb, DciP3, DisplayP3, GammaSrgb, LinAdobeRgb, LinDciP3, LinDisplayP3, LinProPhotoRgb,
    LinRec709, LinRec2020, LinSrgb, ProPhotoRgb, Rec709, Rec2020, Srgb,
};
use palette::white_point::{D50, D65};
use palette::{Hsl, Hsv, Hwb, Lab, Lch, Lchuv, Luv, Oklab, Oklch, Xyz, Yxy};
use quack_rs::error::ExtensionError;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

use super::color::{PltColor, space_arg};
use crate::extension::functions::err;

/// 支持随机采样的空间名（palette 实现了 `Standard` 的那些）。
///
/// The spaces palette can sample from (`Standard` implementations).
pub const RANDOM_SPACES: &[&str] = &[
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
    "hsl",
    "hsv",
    "hwb",
    "lab",
    "lch",
    "luv",
    "lchuv",
    "xyz",
    "xyz_d50",
    "yxy",
    "luma",
    "luma_linear",
    "luma_gamma",
    "oklab",
    "oklch",
    "cam16_ucs_jab",
    "cam16_ucs_jmh",
    "lms",
    "lms_von_kries",
];

/// 用给定 PRNG 采一个该空间的颜色。
///
/// Sample one color in the given space from the supplied PRNG.
fn sample_rng(fn_name: &str, space: &str, rng: &mut StdRng) -> Result<PltColor, ExtensionError> {
    let space = space_arg(fn_name, space)?;
    let (c1, c2, c3) = match space {
        "srgb" => {
            let c: Srgb<f64> = rng.r#gen();
            (c.red, c.green, c.blue)
        }
        "srgb_linear" => {
            let c: LinSrgb<f64> = rng.r#gen();
            (c.red, c.green, c.blue)
        }
        "srgb_gamma" => {
            let c: GammaSrgb<f64> = rng.r#gen();
            (c.red, c.green, c.blue)
        }
        "adobe_rgb" => {
            let c: AdobeRgb<f64> = rng.r#gen();
            (c.red, c.green, c.blue)
        }
        "adobe_rgb_linear" => {
            let c: LinAdobeRgb<f64> = rng.r#gen();
            (c.red, c.green, c.blue)
        }
        "display_p3" => {
            let c: DisplayP3<f64> = rng.r#gen();
            (c.red, c.green, c.blue)
        }
        "display_p3_linear" => {
            let c: LinDisplayP3<f64> = rng.r#gen();
            (c.red, c.green, c.blue)
        }
        "dci_p3" => {
            let c: DciP3<f64> = rng.r#gen();
            (c.red, c.green, c.blue)
        }
        "dci_p3_linear" => {
            let c: LinDciP3<f64> = rng.r#gen();
            (c.red, c.green, c.blue)
        }
        "prophoto_rgb" => {
            let c: ProPhotoRgb<f64> = rng.r#gen();
            (c.red, c.green, c.blue)
        }
        "prophoto_rgb_linear" => {
            let c: LinProPhotoRgb<f64> = rng.r#gen();
            (c.red, c.green, c.blue)
        }
        "rec709" => {
            let c: Rec709<f64> = rng.r#gen();
            (c.red, c.green, c.blue)
        }
        "rec709_linear" => {
            let c: LinRec709<f64> = rng.r#gen();
            (c.red, c.green, c.blue)
        }
        "rec2020" => {
            let c: Rec2020<f64> = rng.r#gen();
            (c.red, c.green, c.blue)
        }
        "rec2020_linear" => {
            let c: LinRec2020<f64> = rng.r#gen();
            (c.red, c.green, c.blue)
        }
        "hsl" => {
            let c: Hsl<SrgbEncoding, f64> = rng.r#gen();
            (c.hue.into_positive_degrees(), c.saturation, c.lightness)
        }
        "hsv" => {
            let c: Hsv<SrgbEncoding, f64> = rng.r#gen();
            (c.hue.into_positive_degrees(), c.saturation, c.value)
        }
        "hwb" => {
            let c: Hwb<SrgbEncoding, f64> = rng.r#gen();
            (c.hue.into_positive_degrees(), c.whiteness, c.blackness)
        }
        "lab" => {
            let c: Lab<D65, f64> = rng.r#gen();
            (c.l, c.a, c.b)
        }
        "lch" => {
            let c: Lch<D65, f64> = rng.r#gen();
            (c.l, c.chroma, c.hue.into_positive_degrees())
        }
        "luv" => {
            let c: Luv<D65, f64> = rng.r#gen();
            (c.l, c.u, c.v)
        }
        "lchuv" => {
            let c: Lchuv<D65, f64> = rng.r#gen();
            (c.l, c.chroma, c.hue.into_positive_degrees())
        }
        "xyz" => {
            let c: Xyz<D65, f64> = rng.r#gen();
            (c.x, c.y, c.z)
        }
        "xyz_d50" => {
            let c: Xyz<D50, f64> = rng.r#gen();
            (c.x, c.y, c.z)
        }
        "yxy" => {
            let c: Yxy<D65, f64> = rng.r#gen();
            (c.x, c.y, c.luma)
        }
        "luma" => {
            let c: Luma<SrgbEncoding, f64> = rng.r#gen();
            (c.luma, 0.0, 0.0)
        }
        "luma_linear" => {
            let c: Luma<Linear<D65>, f64> = rng.r#gen();
            (c.luma, 0.0, 0.0)
        }
        "luma_gamma" => {
            let c: Luma<Gamma<D65>, f64> = rng.r#gen();
            (c.luma, 0.0, 0.0)
        }
        "oklab" => {
            let c: Oklab<f64> = rng.r#gen();
            (c.l, c.a, c.b)
        }
        "oklch" => {
            let c: Oklch<f64> = rng.r#gen();
            (c.l, c.chroma, c.hue.into_positive_degrees())
        }
        "cam16_ucs_jab" => {
            let c: Cam16UcsJab<f64> = rng.r#gen();
            (c.lightness, c.a, c.b)
        }
        "cam16_ucs_jmh" => {
            let c: Cam16UcsJmh<f64> = rng.r#gen();
            (c.lightness, c.colorfulness, c.hue.into_positive_degrees())
        }
        "lms" => {
            let c: BradfordLms<D65, f64> = rng.r#gen();
            (c.long, c.medium, c.short)
        }
        "lms_von_kries" => {
            let c: VonKriesLms<D65, f64> = rng.r#gen();
            (c.long, c.medium, c.short)
        }
        other => {
            return Err(err(format!(
                "{fn_name}: color space '{}' has no random distribution; use one of {}",
                other,
                RANDOM_SPACES.join(", ")
            )));
        }
    };
    Ok(PltColor::new(space, c1, c2, c3, 1.0))
}

/// 采样一个颜色。
///
/// Sample a single color.
pub fn random_color(fn_name: &str, space: &str, seed: u64) -> Result<PltColor, ExtensionError> {
    let mut rng = StdRng::seed_from_u64(seed);
    sample_rng(fn_name, space, &mut rng)
}

/// 用同一个序列采 `count` 个颜色。
///
/// Sample `count` colors from one seeded sequence.
pub fn random_colors(
    fn_name: &str,
    space: &str,
    count: usize,
    seed: u64,
) -> Result<Vec<PltColor>, ExtensionError> {
    let mut rng = StdRng::seed_from_u64(seed);
    let mut colors = Vec::with_capacity(count);
    for _ in 0..count {
        colors.push(sample_rng(fn_name, space, &mut rng)?);
    }
    Ok(colors)
}
