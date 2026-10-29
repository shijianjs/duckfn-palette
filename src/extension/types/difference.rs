// ============================================================================
// 色差（`color_difference` 模块）与 WCAG 2.1 相对对比度（`relative_contrast`）。
//
// 色差的七种度量各自只在特定颜色空间里有定义，因此 `plt_difference` 有一个可选的空间参数：
//   - 不给空间时一律用 CIE L*a*b*（D65），那是 CIEDE2000 / ΔE / HyAb 的规范空间；
//   - 给了空间时先把两个颜色都转过去（未裁剪），再在该空间里算；
//   - 该空间不支持这个度量时报查询错误，并列出可用的空间。
//
//   metric               定义所在空间
//   euclidean            cam16_ucs_jab, lab, lms, luma, luv, oklab, 所有 rgb, xyz, xyz_d50, yxy
//   euclidean_squared    同上
//   delta_e              cam16_ucs_jab, cam16_ucs_jmh, lab, lch
//   improved_delta_e     cam16_ucs_jab, cam16_ucs_jmh, lab, lch
//   hyab                 cam16_ucs_jab, lab, luv, oklab
//   ciede2000            lab, lch
//   improved_ciede2000   lab, lch
//
// 对比度按 WCAG 2.1：先把颜色转到 sRGB（规范假设 sRGB），相对亮度 = 线性 sRGB 的亮度，
// 对比度 = (L亮 + 0.05) / (L暗 + 0.05)，取值 1..21。`plt_contrast_check` 对应标准里的五条
// 判定（正文 4.5、大字号 3、AAA 正文 7、AAA 大字号 4.5、图形 3）。
//
// Color differences (seven metrics, each defined in specific spaces, so the space is an optional
// argument defaulting to CIE L*a*b*) and WCAG 2.1 relative contrast (computed in sRGB).
// ============================================================================

// 同 color.rs：`srgb_gamma` / `luma_gamma` 用的是 palette 标为 deprecated 的 gamma 2.2 类型。
#![allow(deprecated)]

use palette::color_difference::{
    Ciede2000, DeltaE, EuclideanDistance, HyAb, ImprovedCiede2000, ImprovedDeltaE,
    Wcag21RelativeContrast,
};
use palette::encoding;
use palette::lms::{BradfordLms, VonKriesLms};
use palette::luma::Luma;
use palette::rgb::{
    AdobeRgb, DciP3, DisplayP3, GammaSrgb, LinAdobeRgb, LinDciP3, LinDisplayP3, LinProPhotoRgb,
    LinRec2020, LinSrgb, ProPhotoRgb, Rec2020, Rec709, Srgb,
};
use palette::white_point::{D50, D65};
use palette::{Lab, Lch, Luv, Oklab, Xyz, Yxy};
use quack_rs::error::ExtensionError;

use super::color::{PltColor, bad_arg, space_arg};
use super::ops::SqlComponents;

/// 支持 `EuclideanDistance` 的空间。
///
/// Spaces with `EuclideanDistance`.
const EUCLIDEAN_SPACES: &str =
    "cam16_ucs_jab, lab, lms, lms_von_kries, luma, luma_gamma, luma_linear, luv, oklab, every rgb \
     space, xyz, xyz_d50, yxy";

/// 支持 `DeltaE`（以及 `ImprovedDeltaE`）的空间。
///
/// Spaces with `DeltaE` and `ImprovedDeltaE`.
const DELTA_E_SPACES: &str = "cam16_ucs_jab, cam16_ucs_jmh, lab, lch";

/// 支持 `HyAb` 的空间。
///
/// Spaces with `HyAb`.
const HYAB_SPACES: &str = "cam16_ucs_jab, lab, luv, oklab";

/// 支持 `Ciede2000`（以及 `ImprovedCiede2000`）的空间。
///
/// Spaces with `Ciede2000` and `ImprovedCiede2000`.
const CIEDE2000_SPACES: &str = "lab, lch";

/// 所有色差度量名（规范化后）。
///
/// Every color difference metric name, normalized.
pub const METRICS: &[&str] = &[
    "euclidean",
    "euclidean_squared",
    "delta_e",
    "improved_delta_e",
    "hyab",
    "ciede2000",
    "improved_ciede2000",
];

/// WCAG 2.1 的五条对比度判定名（规范化后）。
///
/// The five WCAG 2.1 contrast criteria, normalized.
pub const CONTRAST_CRITERIA: &[&str] = &[
    "min_text",
    "min_large_text",
    "enhanced_text",
    "enhanced_large_text",
    "min_graphics",
];

/// 把名字规范化：小写，`-` 与空格视作 `_`。
///
/// Normalize a name: lowercase, `-` and spaces become `_`.
fn normalize(name: &str) -> String {
    name.trim().to_ascii_lowercase().replace(['-', ' '], "_")
}

/// 解析度量名，接受几个常见别名。
///
/// Resolve a metric name, accepting a couple of common aliases.
fn metric_arg(fn_name: &str, metric: &str) -> Result<&'static str, ExtensionError> {
    let name = normalize(metric);
    let canonical = match name.as_str() {
        "cie76" | "deltae" | "delta_e_76" => "delta_e",
        "cie2000" | "delta_e_2000" => "ciede2000",
        "ciede2000_improved" | "improved_ciede2000" => "improved_ciede2000",
        other => METRICS.iter().copied().find(|m| *m == other).ok_or_else(|| {
            bad_arg(
                fn_name,
                format!(
                    "unknown color difference metric '{metric}'; supported: {}",
                    METRICS.join(", ")
                ),
            )
        })?,
    };
    Ok(canonical)
}

/// 该度量在给定空间里没有定义时的统一错误。
///
/// Uniform "this metric is not defined in that space" error.
fn unsupported_metric(
    fn_name: &str,
    metric: &str,
    space: &str,
    spaces: &str,
) -> ExtensionError {
    bad_arg(
        fn_name,
        format!(
            "{metric} is not defined in color space '{space}'; use one of: {spaces}"
        ),
    )
}

/// `EuclideanDistance` 的分派。
///
/// Dispatch for `EuclideanDistance`.
macro_rules! euclidean_op {
    ($space:expr, $fn_name:expr, $metric:expr, $a:expr, $b:expr, $method:ident) => {{
        let (a, b) = ($a, $b);
        match $space {
            "srgb" => Ok(Srgb::<f64>::from_triple(a).$method(Srgb::<f64>::from_triple(b))),
            "srgb_linear" => Ok(LinSrgb::<f64>::from_triple(a).$method(LinSrgb::<f64>::from_triple(b))),
            "srgb_gamma" => Ok(GammaSrgb::<f64>::from_triple(a).$method(GammaSrgb::<f64>::from_triple(b))),
            "adobe_rgb" => Ok(AdobeRgb::<f64>::from_triple(a).$method(AdobeRgb::<f64>::from_triple(b))),
            "adobe_rgb_linear" => Ok(LinAdobeRgb::<f64>::from_triple(a).$method(LinAdobeRgb::<f64>::from_triple(b))),
            "display_p3" => Ok(DisplayP3::<f64>::from_triple(a).$method(DisplayP3::<f64>::from_triple(b))),
            "display_p3_linear" => Ok(LinDisplayP3::<f64>::from_triple(a).$method(LinDisplayP3::<f64>::from_triple(b))),
            "dci_p3" => Ok(DciP3::<f64>::from_triple(a).$method(DciP3::<f64>::from_triple(b))),
            "dci_p3_linear" => Ok(LinDciP3::<f64>::from_triple(a).$method(LinDciP3::<f64>::from_triple(b))),
            "prophoto_rgb" => Ok(ProPhotoRgb::<f64>::from_triple(a).$method(ProPhotoRgb::<f64>::from_triple(b))),
            "prophoto_rgb_linear" => Ok(LinProPhotoRgb::<f64>::from_triple(a).$method(LinProPhotoRgb::<f64>::from_triple(b))),
            "rec709" => Ok(Rec709::<f64>::from_triple(a).$method(Rec709::<f64>::from_triple(b))),
            "rec709_linear" => Ok(LinSrgb::<f64>::from_triple(a).$method(LinSrgb::<f64>::from_triple(b))),
            "rec2020" => Ok(Rec2020::<f64>::from_triple(a).$method(Rec2020::<f64>::from_triple(b))),
            "rec2020_linear" => Ok(LinRec2020::<f64>::from_triple(a).$method(LinRec2020::<f64>::from_triple(b))),
            "lab" => Ok(Lab::<D65, f64>::from_triple(a).$method(Lab::<D65, f64>::from_triple(b))),
            "luv" => Ok(Luv::<D65, f64>::from_triple(a).$method(Luv::<D65, f64>::from_triple(b))),
            "xyz" => Ok(Xyz::<D65, f64>::from_triple(a).$method(Xyz::<D65, f64>::from_triple(b))),
            "xyz_d50" => Ok(Xyz::<D50, f64>::from_triple(a).$method(Xyz::<D50, f64>::from_triple(b))),
            "yxy" => Ok(Yxy::<D65, f64>::from_triple(a).$method(Yxy::<D65, f64>::from_triple(b))),
            "luma" => Ok(Luma::<encoding::Srgb, f64>::from_triple(a).$method(Luma::<encoding::Srgb, f64>::from_triple(b))),
            "luma_linear" => Ok(Luma::<encoding::Linear<D65>, f64>::from_triple(a).$method(Luma::<encoding::Linear<D65>, f64>::from_triple(b))),
            "luma_gamma" => Ok(Luma::<encoding::Gamma<D65>, f64>::from_triple(a).$method(Luma::<encoding::Gamma<D65>, f64>::from_triple(b))),
            "oklab" => Ok(Oklab::<f64>::from_triple(a).$method(Oklab::<f64>::from_triple(b))),
            "cam16_ucs_jab" => Ok(palette::cam16::Cam16UcsJab::<f64>::from_triple(a).$method(palette::cam16::Cam16UcsJab::<f64>::from_triple(b))),
            "lms" => Ok(BradfordLms::<D65, f64>::from_triple(a).$method(BradfordLms::<D65, f64>::from_triple(b))),
            "lms_von_kries" => Ok(VonKriesLms::<D65, f64>::from_triple(a).$method(VonKriesLms::<D65, f64>::from_triple(b))),
            other => Err(unsupported_metric($fn_name, $metric, other, EUCLIDEAN_SPACES)),
        }
    }};
}

/// `DeltaE` / `ImprovedDeltaE` 的分派。
///
/// Dispatch for `DeltaE` and `ImprovedDeltaE`.
macro_rules! delta_e_op {
    ($space:expr, $fn_name:expr, $metric:expr, $a:expr, $b:expr, $method:ident) => {{
        let (a, b) = ($a, $b);
        match $space {
            "lab" => Ok(Lab::<D65, f64>::from_triple(a).$method(Lab::<D65, f64>::from_triple(b))),
            "lch" => Ok(Lch::<D65, f64>::from_triple(a).$method(Lch::<D65, f64>::from_triple(b))),
            "cam16_ucs_jab" => Ok(palette::cam16::Cam16UcsJab::<f64>::from_triple(a).$method(palette::cam16::Cam16UcsJab::<f64>::from_triple(b))),
            "cam16_ucs_jmh" => Ok(palette::cam16::Cam16UcsJmh::<f64>::from_triple(a).$method(palette::cam16::Cam16UcsJmh::<f64>::from_triple(b))),
            other => Err(unsupported_metric($fn_name, $metric, other, DELTA_E_SPACES)),
        }
    }};
}

/// `HyAb` 的分派。
///
/// Dispatch for `HyAb`.
macro_rules! hyab_op {
    ($space:expr, $fn_name:expr, $metric:expr, $a:expr, $b:expr) => {{
        let (a, b) = ($a, $b);
        match $space {
            "lab" => Ok(Lab::<D65, f64>::from_triple(a).hybrid_distance(Lab::<D65, f64>::from_triple(b))),
            "luv" => Ok(Luv::<D65, f64>::from_triple(a).hybrid_distance(Luv::<D65, f64>::from_triple(b))),
            "oklab" => Ok(Oklab::<f64>::from_triple(a).hybrid_distance(Oklab::<f64>::from_triple(b))),
            "cam16_ucs_jab" => Ok(palette::cam16::Cam16UcsJab::<f64>::from_triple(a).hybrid_distance(palette::cam16::Cam16UcsJab::<f64>::from_triple(b))),
            other => Err(unsupported_metric($fn_name, $metric, other, HYAB_SPACES)),
        }
    }};
}

/// `Ciede2000` / `ImprovedCiede2000` 的分派。
///
/// Dispatch for `Ciede2000` and `ImprovedCiede2000`.
macro_rules! ciede2000_op {
    ($space:expr, $fn_name:expr, $metric:expr, $a:expr, $b:expr, $method:ident) => {{
        let (a, b) = ($a, $b);
        match $space {
            "lab" => Ok(Lab::<D65, f64>::from_triple(a).$method(Lab::<D65, f64>::from_triple(b))),
            "lch" => Ok(Lch::<D65, f64>::from_triple(a).$method(Lch::<D65, f64>::from_triple(b))),
            other => Err(unsupported_metric($fn_name, $metric, other, CIEDE2000_SPACES)),
        }
    }};
}

impl PltColor {
    /// 两个颜色在指定空间里的色差；`space` 为空时用 CIE L*a*b*（D65）。
    ///
    /// The color difference between two colors in the given space, or CIE L*a*b* (D65) by default.
    pub fn difference(
        &self,
        other: &Self,
        metric: &str,
        space: Option<&str>,
        fn_name: &str,
    ) -> Result<f64, ExtensionError> {
        let metric = metric_arg(fn_name, metric)?;
        let target = match space {
            Some(space) => space_arg(fn_name, space)?,
            None => "lab",
        };
        let a = self.convert(fn_name, target)?;
        let b = other.convert(fn_name, target)?;
        let ta = (a.c1, a.c2, a.c3);
        let tb = (b.c1, b.c2, b.c3);
        match metric {
            "euclidean" => euclidean_op!(target, fn_name, metric, ta, tb, distance),
            "euclidean_squared" => euclidean_op!(target, fn_name, metric, ta, tb, distance_squared),
            "delta_e" => delta_e_op!(target, fn_name, metric, ta, tb, delta_e),
            "improved_delta_e" => delta_e_op!(target, fn_name, metric, ta, tb, improved_delta_e),
            "hyab" => hyab_op!(target, fn_name, metric, ta, tb),
            "ciede2000" => ciede2000_op!(target, fn_name, metric, ta, tb, difference),
            "improved_ciede2000" => {
                ciede2000_op!(target, fn_name, metric, ta, tb, improved_difference)
            }
            _ => unreachable!("metric_arg returned an unknown metric"),
        }
    }

    /// 转到 sRGB（WCAG 的假设空间）。
    ///
    /// Convert into sRGB, the space WCAG assumes.
    fn to_wcag_srgb(&self, fn_name: &str) -> Result<Srgb<f64>, ExtensionError> {
        let c = self.convert(fn_name, "srgb")?;
        Ok(Srgb::new(c.c1, c.c2, c.c3))
    }

    /// WCAG 2.1 相对亮度（0 黑 .. 1 白）。
    ///
    /// The WCAG 2.1 relative luminance (0 is black, 1 is white).
    pub fn relative_luminance(&self, fn_name: &str) -> Result<f64, ExtensionError> {
        let rgb = self.to_wcag_srgb(fn_name)?;
        Ok(rgb.relative_luminance().luma)
    }

    /// WCAG 2.1 相对对比度（1..21）。
    ///
    /// The WCAG 2.1 relative contrast ratio (1..21).
    pub fn contrast_ratio(&self, other: &Self, fn_name: &str) -> Result<f64, ExtensionError> {
        let a = self.to_wcag_srgb(fn_name)?;
        let b = other.to_wcag_srgb(fn_name)?;
        Ok(a.relative_contrast(b))
    }

    /// 是否满足 WCAG 2.1 的某条对比度判定。
    ///
    /// Whether the pair meets one of the WCAG 2.1 contrast criteria.
    pub fn contrast_meets(
        &self,
        other: &Self,
        criterion: &str,
        fn_name: &str,
    ) -> Result<bool, ExtensionError> {
        let name = normalize(criterion);
        let criterion = CONTRAST_CRITERIA
            .iter()
            .copied()
            .find(|c| *c == name)
            .ok_or_else(|| {
                bad_arg(
                    fn_name,
                    format!(
                        "unknown contrast criterion '{criterion}'; supported: {}",
                        CONTRAST_CRITERIA.join(", ")
                    ),
                )
            })?;
        let a = self.to_wcag_srgb(fn_name)?;
        let b = other.to_wcag_srgb(fn_name)?;
        let meets = match criterion {
            "min_text" => a.has_min_contrast_text(b),
            "min_large_text" => a.has_min_contrast_large_text(b),
            "enhanced_text" => a.has_enhanced_contrast_text(b),
            "enhanced_large_text" => a.has_enhanced_contrast_large_text(b),
            "min_graphics" => a.has_min_contrast_graphics(b),
            _ => unreachable!("criterion was resolved from CONTRAST_CRITERIA"),
        };
        Ok(meets)
    }
}
