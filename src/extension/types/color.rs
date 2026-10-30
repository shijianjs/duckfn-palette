// ============================================================================
// 统一的颜色值 `plt_color`，以及它和 palette 各颜色类型之间的转换核心。
//
// SQL 里没有「颜色空间」这种静态类型，所以这里把 palette 的 `Rgb<S,T>` / `Lab<Wp,T>` /
// `Cam16Jch<T>` 等等擦除成一个带空间标签的 STRUCT：
//
//     STRUCT(space VARCHAR, c1 DOUBLE, c2 DOUBLE, c3 DOUBLE, alpha DOUBLE)
//
// `space` 是规范化后的空间名（见 `SPACES` / `canonical_space`），`c1..c3` 是该空间的
// 三个分量（顺序与 palette 的 `new` 完全一致；一维空间如 luma 只用 c1，其余填 0），
// `alpha` 与 palette 的 `Alpha` 同义（0 透明、1 不透明；转换不会改动它）。
//
// 所有跨空间转换都以 XYZ(D65) 为中转，这是 palette 自己的转换图也有的枢纽：
//   - 各空间 → XYZ(D65)：编码 RGB 先解传输函数，再走 Rgb→Xyz；D50 的 ProPhoto 与
//     `xyz_d50` 用 Bradford 适应到 D65；DCI-P3 的白点类型没实现 `HasXyzMeta`，
//     用一个转发它的本地白点 `DciP3WhitePoint` 走同一套适应；
//   - XYZ(D65) → 各空间：对称地反向，必要时先适应到目标白点。
//   - 中间空间（Lab/Luv/Oklab/Lch/Hsluv/Hwb…）按 palette 的转换图逐跳转换，
//     不存在直达 impl 的地方就显式写两跳。
// CAM16 没有中转类型，直接 `from_xyz` / `into_xyz`，使用 palette 的默认观察条件
// （D65、La=40、Yb=0.2、average surround、auto discounting）；需要自定义观察条件时用
// `cam16` 模块里单列的函数。
//
// The unified SQL color value and the conversion core between it and palette's typed colors.
// A tagged STRUCT erases palette's type parameters; every conversion goes through XYZ(D65),
// the hub palette's own conversion graph has. CAM16 converts directly with palette's default
// viewing conditions; the cam16 module exposes explicit conditions.
// ============================================================================

// 刻意暴露 palette 的 `GammaSrgb` / `GammaLuma`（gamma 2.2 编码），它们被 palette 标记为
// deprecated 但仍是公开能力；SQL 侧保留 `srgb_gamma` / `luma_gamma` 两个空间名。
// We deliberately expose palette's deprecated gamma 2.2 spaces as `srgb_gamma` / `luma_gamma`.
#![allow(deprecated)]

use duckfn::{DuckStruct, duck_error};
use palette::cam16::{
    Cam16, Cam16Jch, Cam16Jmh, Cam16Jsh, Cam16Qch, Cam16Qmh, Cam16Qsh, Cam16UcsJab, Cam16UcsJmh,
    Parameters, StaticWp,
};
use palette::chromatic_adaptation::AdaptFromUnclamped;
use palette::convert::FromColorUnclamped;
use palette::lms::{BradfordLms, VonKriesLms};
use palette::luma::Luma;
use palette::num::Real;
use palette::rgb::{
    AdobeRgb, DciP3, DisplayP3, GammaSrgb, LinAdobeRgb, LinDciP3, LinDisplayP3, LinProPhotoRgb,
    LinRec2020, LinSrgb, ProPhotoRgb, Rec709, Rec2020, Srgb,
};
use palette::white_point::{Any, D50, D65, WhitePoint};
use palette::xyz::meta::HasXyzMeta;
use palette::{
    Hsl, Hsluv, Hsv, Hwb, Lab, Lch, Lchuv, Luv, Okhsl, Okhsv, Okhwb, Oklab, Oklch, Xyz, Yxy,
};
use quack_rs::error::ExtensionError;

use crate::extension::functions::err;

/// SQL 侧的颜色值类型：空间标签 + 三个分量 + alpha。
///
/// The SQL-facing color value: space tag, three components and alpha.
#[derive(Debug, Clone, DuckStruct)]
#[duck(sql_name = "plt_color", create_type = true)]
pub struct PltColor {
    pub space: String,
    pub c1: f64,
    pub c2: f64,
    pub c3: f64,
    pub alpha: f64,
}

impl Default for PltColor {
    fn default() -> Self {
        Self::new("srgb", 0.0, 0.0, 0.0, 1.0)
    }
}

/// 所有受支持空间名（规范化形式），顺序即文档里的展示顺序。
///
/// Every supported space name, in documentation order.
pub const SPACES: &[&str] = &[
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
    "hsluv",
    "xyz",
    "xyz_d50",
    "yxy",
    "luma",
    "luma_linear",
    "luma_gamma",
    "oklab",
    "oklch",
    "okhsl",
    "okhsv",
    "okhwb",
    "cam16_jch",
    "cam16_jmh",
    "cam16_jsh",
    "cam16_qch",
    "cam16_qmh",
    "cam16_qsh",
    "cam16_ucs_jab",
    "cam16_ucs_jmh",
    "lms",
    "lms_von_kries",
];

/// 把用户输入的空间名规范化为 `SPACES` 里的形式：大小写不敏感，`-` / 空格视作 `_`，
/// 并接受一组常见别名（`xyz-d65`、`p3`、`prophoto`、`cam16` 等）。
///
/// Normalize a user-supplied space name; case-insensitive, treats `-` and spaces as `_`, and
/// accepts a few common aliases.
pub fn canonical_space(name: &str) -> Option<&'static str> {
    let normalized = name.trim().to_ascii_lowercase().replace(['-', ' '], "_");
    let canonical = match normalized.as_str() {
        "xyz" | "xyz_d65" | "cie_xyz" => "xyz",
        "linear_srgb" | "srgb_lin" => "srgb_linear",
        "srgb_gamma" | "gamma_srgb" => "srgb_gamma",
        "linear_adobe_rgb" | "adobe_linear" => "adobe_rgb_linear",
        "p3" | "displayp3" | "display_p3" => "display_p3",
        "linear_display_p3" | "display_p3_linear" => "display_p3_linear",
        "dcip3" | "dci_p3" => "dci_p3",
        "linear_dci_p3" | "dci_p3_linear" => "dci_p3_linear",
        "prophoto" | "prophoto_rgb" => "prophoto_rgb",
        "linear_prophoto" | "prophoto_rgb_linear" => "prophoto_rgb_linear",
        "rec_709" | "rec709" => "rec709",
        "linear_rec_709" | "rec709_linear" => "rec709_linear",
        "rec_2020" | "rec2020" => "rec2020",
        "linear_rec_2020" | "rec2020_linear" => "rec2020_linear",
        "cie_lab" | "lab" => "lab",
        "cie_lch" | "lch" => "lch",
        "cie_luv" | "luv" => "luv",
        "cie_lchuv" | "lchuv" => "lchuv",
        "hsluv" => "hsluv",
        "xyz_d50" => "xyz_d50",
        "xyy" | "yxy" => "yxy",
        "grey" | "gray" | "luma" => "luma",
        "linear_luma" | "luma_linear" => "luma_linear",
        "gamma_luma" | "luma_gamma" => "luma_gamma",
        "ok_lab" | "oklab" => "oklab",
        "ok_lch" | "oklch" => "oklch",
        "ok_hsl" | "okhsl" => "okhsl",
        "ok_hsv" | "okhsv" => "okhsv",
        "ok_hwb" | "okhwb" => "okhwb",
        "cam16" | "cam16_jch" => "cam16_jch",
        "cam16_jmh" => "cam16_jmh",
        "cam16_jsh" => "cam16_jsh",
        "cam16_qch" => "cam16_qch",
        "cam16_qmh" => "cam16_qmh",
        "cam16_qsh" => "cam16_qsh",
        "cam16_ucs" | "cam16_ucs_jab" => "cam16_ucs_jab",
        "cam16_ucs_jmh" => "cam16_ucs_jmh",
        "lms" | "bradford_lms" => "lms",
        "von_kries_lms" | "lms_von_kries" => "lms_von_kries",
        _ => SPACES.iter().copied().find(|s| *s == normalized)?,
    };
    Some(canonical)
}

/// DCI-P3 的影院白点（~6300K）。palette 的 `encoding::DciP3` 实现了 `WhitePoint`，但没实现
/// 色适应所需的 `HasXyzMeta`；这里转发它的数值，让 Bradford 适应可用。
///
/// DCI-P3's theatrical white point. palette's `encoding::DciP3` implements `WhitePoint` but not
/// `HasXyzMeta`; this forwards it so Bradford adaptation works.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct DciP3WhitePoint;

impl<T: Real> WhitePoint<T> for DciP3WhitePoint {
    fn get_xyz() -> Xyz<Any, T> {
        <palette::encoding::DciP3 as WhitePoint<T>>::get_xyz()
    }
}

impl HasXyzMeta for DciP3WhitePoint {
    type XyzMeta = Self;
}

/// palette 的颜色值（类型已确定，但 alpha 由 `PltColor` 单独持有）。
///
/// A palette color with a concrete type; alpha stays on the `PltColor` wrapping it.
#[derive(Debug, Clone, Copy)]
pub enum AnyColor {
    Srgb(Srgb<f64>),
    SrgbLinear(LinSrgb<f64>),
    SrgbGamma(GammaSrgb<f64>),
    AdobeRgb(AdobeRgb<f64>),
    AdobeRgbLinear(LinAdobeRgb<f64>),
    DisplayP3(DisplayP3<f64>),
    DisplayP3Linear(LinDisplayP3<f64>),
    DciP3(DciP3<f64>),
    DciP3Linear(LinDciP3<f64>),
    ProPhoto(ProPhotoRgb<f64>),
    ProPhotoLinear(LinProPhotoRgb<f64>),
    Rec709(Rec709<f64>),
    Rec2020(Rec2020<f64>),
    Rec2020Linear(LinRec2020<f64>),
    Hsl(Hsl<palette::encoding::Srgb, f64>),
    Hsv(Hsv<palette::encoding::Srgb, f64>),
    Hwb(Hwb<palette::encoding::Srgb, f64>),
    Lab(Lab<D65, f64>),
    Lch(Lch<D65, f64>),
    Luv(Luv<D65, f64>),
    Lchuv(Lchuv<D65, f64>),
    Hsluv(Hsluv<D65, f64>),
    Xyz(Xyz<D65, f64>),
    XyzD50(Xyz<D50, f64>),
    Yxy(Yxy<D65, f64>),
    Luma(Luma<palette::encoding::Srgb, f64>),
    LumaLinear(Luma<palette::encoding::Linear<D65>, f64>),
    LumaGamma(Luma<palette::encoding::Gamma<D65>, f64>),
    Oklab(Oklab<f64>),
    Oklch(Oklch<f64>),
    Okhsl(Okhsl<f64>),
    Okhsv(Okhsv<f64>),
    Okhwb(Okhwb<f64>),
    Cam16Jch(Cam16Jch<f64>),
    Cam16Jmh(Cam16Jmh<f64>),
    Cam16Jsh(Cam16Jsh<f64>),
    Cam16Qch(Cam16Qch<f64>),
    Cam16Qmh(Cam16Qmh<f64>),
    Cam16Qsh(Cam16Qsh<f64>),
    Cam16UcsJab(Cam16UcsJab<f64>),
    Cam16UcsJmh(Cam16UcsJmh<f64>),
    Lms(BradfordLms<D65, f64>),
    LmsVonKries(VonKriesLms<D65, f64>),
}

impl PltColor {
    pub fn new(space: &str, c1: f64, c2: f64, c3: f64, alpha: f64) -> Self {
        Self {
            space: space.to_string(),
            c1,
            c2,
            c3,
            alpha,
        }
    }

    /// 规范化空间名；不认识时返回以函数名开头的错误。
    ///
    /// Normalize the space name, or fail with a message that starts with the function name.
    pub fn require_space(&self, fn_name: &str) -> Result<&'static str, ExtensionError> {
        space_arg(fn_name, &self.space)
    }

    /// 从 SQL 值解出 palette 的强类型颜色。
    ///
    /// Decode the SQL value into a typed palette color.
    pub fn any(&self) -> Result<AnyColor, ExtensionError> {
        self.any_as("plt_color")
    }

    pub fn any_as(&self, fn_name: &str) -> Result<AnyColor, ExtensionError> {
        let space = self.require_space(fn_name)?;
        let (c1, c2, c3) = (self.c1, self.c2, self.c3);
        let color = match space {
            "srgb" => AnyColor::Srgb(Srgb::new(c1, c2, c3)),
            "srgb_linear" => AnyColor::SrgbLinear(LinSrgb::new(c1, c2, c3)),
            "srgb_gamma" => AnyColor::SrgbGamma(GammaSrgb::new(c1, c2, c3)),
            "adobe_rgb" => AnyColor::AdobeRgb(AdobeRgb::new(c1, c2, c3)),
            "adobe_rgb_linear" => AnyColor::AdobeRgbLinear(LinAdobeRgb::new(c1, c2, c3)),
            "display_p3" => AnyColor::DisplayP3(DisplayP3::new(c1, c2, c3)),
            "display_p3_linear" => AnyColor::DisplayP3Linear(LinDisplayP3::new(c1, c2, c3)),
            "dci_p3" => AnyColor::DciP3(DciP3::new(c1, c2, c3)),
            "dci_p3_linear" => AnyColor::DciP3Linear(LinDciP3::new(c1, c2, c3)),
            "prophoto_rgb" => AnyColor::ProPhoto(ProPhotoRgb::new(c1, c2, c3)),
            "prophoto_rgb_linear" => AnyColor::ProPhotoLinear(LinProPhotoRgb::new(c1, c2, c3)),
            "rec709" => AnyColor::Rec709(Rec709::new(c1, c2, c3)),
            "rec709_linear" => AnyColor::SrgbLinear(LinSrgb::new(c1, c2, c3)),
            "rec2020" => AnyColor::Rec2020(Rec2020::new(c1, c2, c3)),
            "rec2020_linear" => AnyColor::Rec2020Linear(LinRec2020::new(c1, c2, c3)),
            "hsl" => AnyColor::Hsl(Hsl::new(c1, c2, c3)),
            "hsv" => AnyColor::Hsv(Hsv::new(c1, c2, c3)),
            "hwb" => AnyColor::Hwb(Hwb::new(c1, c2, c3)),
            "lab" => AnyColor::Lab(Lab::new(c1, c2, c3)),
            "lch" => AnyColor::Lch(Lch::new(c1, c2, c3)),
            "luv" => AnyColor::Luv(Luv::new(c1, c2, c3)),
            "lchuv" => AnyColor::Lchuv(Lchuv::new(c1, c2, c3)),
            "hsluv" => AnyColor::Hsluv(Hsluv::new(c1, c2, c3)),
            "xyz" => AnyColor::Xyz(Xyz::new(c1, c2, c3)),
            "xyz_d50" => AnyColor::XyzD50(Xyz::new(c1, c2, c3)),
            "yxy" => AnyColor::Yxy(Yxy::new(c1, c2, c3)),
            "luma" => AnyColor::Luma(Luma::new(c1)),
            "luma_linear" => AnyColor::LumaLinear(Luma::new(c1)),
            "luma_gamma" => AnyColor::LumaGamma(Luma::new(c1)),
            "oklab" => AnyColor::Oklab(Oklab::new(c1, c2, c3)),
            "oklch" => AnyColor::Oklch(Oklch::new(c1, c2, c3)),
            "okhsl" => AnyColor::Okhsl(Okhsl::new(c1, c2, c3)),
            "okhsv" => AnyColor::Okhsv(Okhsv::new(c1, c2, c3)),
            "okhwb" => AnyColor::Okhwb(Okhwb::new(c1, c2, c3)),
            "cam16_jch" => AnyColor::Cam16Jch(Cam16Jch::new(c1, c2, c3)),
            "cam16_jmh" => AnyColor::Cam16Jmh(Cam16Jmh::new(c1, c2, c3)),
            "cam16_jsh" => AnyColor::Cam16Jsh(Cam16Jsh::new(c1, c2, c3)),
            "cam16_qch" => AnyColor::Cam16Qch(Cam16Qch::new(c1, c2, c3)),
            "cam16_qmh" => AnyColor::Cam16Qmh(Cam16Qmh::new(c1, c2, c3)),
            "cam16_qsh" => AnyColor::Cam16Qsh(Cam16Qsh::new(c1, c2, c3)),
            "cam16_ucs_jab" => AnyColor::Cam16UcsJab(Cam16UcsJab::new(c1, c2, c3)),
            "cam16_ucs_jmh" => AnyColor::Cam16UcsJmh(Cam16UcsJmh::new(c1, c2, c3)),
            "lms" => AnyColor::Lms(BradfordLms::<D65, f64>::new(c1, c2, c3)),
            "lms_von_kries" => AnyColor::LmsVonKries(VonKriesLms::<D65, f64>::new(c1, c2, c3)),
            _ => unreachable!("canonical_space returned an unknown space"),
        };
        Ok(color)
    }

    /// palette 默认的 CAM16 观察条件（D65、La=40、Yb=0.2、average、auto）。
    ///
    /// Palette's default CAM16 viewing conditions.
    fn cam16_parameters() -> Parameters<StaticWp<D65>, f64> {
        Parameters::default_static_wp(40.0)
    }

    /// 任意空间 → XYZ(D65)。
    ///
    /// Any space into XYZ(D65).
    pub fn to_xyz(&self) -> Result<Xyz<D65, f64>, ExtensionError> {
        let xyz = match self.any()? {
            AnyColor::Srgb(c) => Xyz::<D65, f64>::from_color_unclamped(c),
            AnyColor::SrgbLinear(c) => Xyz::<D65, f64>::from_color_unclamped(c),
            AnyColor::SrgbGamma(c) => Xyz::<D65, f64>::from_color_unclamped(c),
            AnyColor::AdobeRgb(c) => Xyz::<D65, f64>::from_color_unclamped(c),
            AnyColor::AdobeRgbLinear(c) => Xyz::<D65, f64>::from_color_unclamped(c),
            AnyColor::DisplayP3(c) => Xyz::<D65, f64>::from_color_unclamped(c),
            AnyColor::DisplayP3Linear(c) => Xyz::<D65, f64>::from_color_unclamped(c),
            AnyColor::DciP3(c) => {
                let xyz: Xyz<palette::encoding::DciP3, f64> = Xyz::from_color_unclamped(c);
                Xyz::<D65, f64>::adapt_from_unclamped(xyz.with_white_point::<DciP3WhitePoint>())
            }
            AnyColor::DciP3Linear(c) => {
                let xyz: Xyz<palette::encoding::DciP3, f64> = Xyz::from_color_unclamped(c);
                Xyz::<D65, f64>::adapt_from_unclamped(xyz.with_white_point::<DciP3WhitePoint>())
            }
            AnyColor::ProPhoto(c) => {
                let xyz: Xyz<D50, f64> = Xyz::from_color_unclamped(c);
                Xyz::<D65, f64>::adapt_from_unclamped(xyz)
            }
            AnyColor::ProPhotoLinear(c) => {
                let xyz: Xyz<D50, f64> = Xyz::from_color_unclamped(c);
                Xyz::<D65, f64>::adapt_from_unclamped(xyz)
            }
            AnyColor::Rec709(c) => Xyz::<D65, f64>::from_color_unclamped(c),
            AnyColor::Rec2020(c) => Xyz::<D65, f64>::from_color_unclamped(c),
            AnyColor::Rec2020Linear(c) => Xyz::<D65, f64>::from_color_unclamped(c),
            AnyColor::Hsl(c) => {
                let rgb = Srgb::from_color_unclamped(c);
                Xyz::<D65, f64>::from_color_unclamped(rgb)
            }
            AnyColor::Hsv(c) => {
                let rgb = Srgb::from_color_unclamped(c);
                Xyz::<D65, f64>::from_color_unclamped(rgb)
            }
            AnyColor::Hwb(c) => {
                let hsv = Hsv::from_color_unclamped(c);
                let rgb = Srgb::from_color_unclamped(hsv);
                Xyz::<D65, f64>::from_color_unclamped(rgb)
            }
            AnyColor::Lab(c) => Xyz::<D65, f64>::from_color_unclamped(c),
            AnyColor::Lch(c) => {
                let lab = Lab::from_color_unclamped(c);
                Xyz::<D65, f64>::from_color_unclamped(lab)
            }
            AnyColor::Luv(c) => Xyz::<D65, f64>::from_color_unclamped(c),
            AnyColor::Lchuv(c) => {
                let luv = Luv::from_color_unclamped(c);
                Xyz::<D65, f64>::from_color_unclamped(luv)
            }
            AnyColor::Hsluv(c) => {
                let lchuv = Lchuv::from_color_unclamped(c);
                let luv = Luv::from_color_unclamped(lchuv);
                Xyz::<D65, f64>::from_color_unclamped(luv)
            }
            AnyColor::Xyz(c) => c,
            AnyColor::XyzD50(c) => Xyz::<D65, f64>::adapt_from_unclamped(c),
            AnyColor::Yxy(c) => Xyz::<D65, f64>::from_color_unclamped(c),
            AnyColor::Luma(c) => Xyz::<D65, f64>::from_color_unclamped(c),
            AnyColor::LumaLinear(c) => Xyz::<D65, f64>::from_color_unclamped(c),
            AnyColor::LumaGamma(c) => Xyz::<D65, f64>::from_color_unclamped(c),
            AnyColor::Oklab(c) => Xyz::<D65, f64>::from_color_unclamped(c),
            AnyColor::Oklch(c) => {
                let lab = Oklab::from_color_unclamped(c);
                Xyz::<D65, f64>::from_color_unclamped(lab)
            }
            AnyColor::Okhsl(c) => {
                let lab = Oklab::from_color_unclamped(c);
                Xyz::<D65, f64>::from_color_unclamped(lab)
            }
            AnyColor::Okhsv(c) => {
                let lab = Oklab::from_color_unclamped(c);
                Xyz::<D65, f64>::from_color_unclamped(lab)
            }
            AnyColor::Okhwb(c) => {
                let hsv = Okhsv::from_color_unclamped(c);
                let lab = Oklab::from_color_unclamped(hsv);
                Xyz::<D65, f64>::from_color_unclamped(lab)
            }
            AnyColor::Cam16Jch(c) => c.into_xyz(Self::cam16_parameters().bake()),
            AnyColor::Cam16Jmh(c) => c.into_xyz(Self::cam16_parameters().bake()),
            AnyColor::Cam16Jsh(c) => c.into_xyz(Self::cam16_parameters().bake()),
            AnyColor::Cam16Qch(c) => c.into_xyz(Self::cam16_parameters().bake()),
            AnyColor::Cam16Qmh(c) => c.into_xyz(Self::cam16_parameters().bake()),
            AnyColor::Cam16Qsh(c) => c.into_xyz(Self::cam16_parameters().bake()),
            AnyColor::Cam16UcsJab(c) => {
                let jmh = Cam16UcsJmh::from_color_unclamped(c);
                let partial = Cam16Jmh::from_color_unclamped(jmh);
                partial.into_xyz(Self::cam16_parameters().bake())
            }
            AnyColor::Cam16UcsJmh(c) => {
                let partial = Cam16Jmh::from_color_unclamped(c);
                partial.into_xyz(Self::cam16_parameters().bake())
            }
            AnyColor::Lms(c) => Xyz::<D65, f64>::from_color_unclamped(c),
            AnyColor::LmsVonKries(c) => Xyz::<D65, f64>::from_color_unclamped(c),
        };
        Ok(xyz)
    }

    /// 把 XYZ(D65) 写成目标空间；alpha 原样带入。
    ///
    /// Encode XYZ(D65) as the target space, carrying alpha along.
    pub fn from_xyz(
        fn_name: &str,
        space: &str,
        xyz: Xyz<D65, f64>,
        alpha: f64,
    ) -> Result<Self, ExtensionError> {
        let space = space_arg(fn_name, space)?;
        let (c1, c2, c3) = match space {
            "srgb" => Srgb::from_color_unclamped(xyz).into_components(),
            "srgb_linear" => LinSrgb::from_color_unclamped(xyz).into_components(),
            "srgb_gamma" => GammaSrgb::from_color_unclamped(xyz).into_components(),
            "adobe_rgb" => AdobeRgb::from_color_unclamped(xyz).into_components(),
            "adobe_rgb_linear" => LinAdobeRgb::from_color_unclamped(xyz).into_components(),
            "display_p3" => DisplayP3::from_color_unclamped(xyz).into_components(),
            "display_p3_linear" => LinDisplayP3::from_color_unclamped(xyz).into_components(),
            "dci_p3" => {
                let adapted: Xyz<DciP3WhitePoint, f64> = Xyz::adapt_from_unclamped(xyz);
                let native = adapted.with_white_point::<palette::encoding::DciP3>();
                DciP3::from_color_unclamped(native).into_components()
            }
            "dci_p3_linear" => {
                let adapted: Xyz<DciP3WhitePoint, f64> = Xyz::adapt_from_unclamped(xyz);
                let native = adapted.with_white_point::<palette::encoding::DciP3>();
                LinDciP3::from_color_unclamped(native).into_components()
            }
            "prophoto_rgb" => {
                let adapted: Xyz<D50, f64> = Xyz::adapt_from_unclamped(xyz);
                ProPhotoRgb::from_color_unclamped(adapted).into_components()
            }
            "prophoto_rgb_linear" => {
                let adapted: Xyz<D50, f64> = Xyz::adapt_from_unclamped(xyz);
                LinProPhotoRgb::from_color_unclamped(adapted).into_components()
            }
            "rec709" => Rec709::from_color_unclamped(xyz).into_components(),
            "rec709_linear" => LinSrgb::from_color_unclamped(xyz).into_components(),
            "rec2020" => Rec2020::from_color_unclamped(xyz).into_components(),
            "rec2020_linear" => LinRec2020::from_color_unclamped(xyz).into_components(),
            "hsl" => {
                let rgb = Srgb::from_color_unclamped(xyz);
                hue_first(Hsl::from_color_unclamped(rgb))
            }
            "hsv" => {
                let rgb = Srgb::from_color_unclamped(xyz);
                let c = Hsv::from_color_unclamped(rgb);
                (c.hue.into_positive_degrees(), c.saturation, c.value)
            }
            "hwb" => {
                let rgb = Srgb::from_color_unclamped(xyz);
                let hsv = Hsv::from_color_unclamped(rgb);
                let c = Hwb::from_color_unclamped(hsv);
                (c.hue.into_positive_degrees(), c.whiteness, c.blackness)
            }
            "lab" => Lab::from_color_unclamped(xyz).into_components(),
            "lch" => {
                let lab = Lab::from_color_unclamped(xyz);
                let c = Lch::from_color_unclamped(lab);
                (c.l, c.chroma, c.hue.into_positive_degrees())
            }
            "luv" => Luv::from_color_unclamped(xyz).into_components(),
            "lchuv" => {
                let luv = Luv::from_color_unclamped(xyz);
                let c = Lchuv::from_color_unclamped(luv);
                (c.l, c.chroma, c.hue.into_positive_degrees())
            }
            "hsluv" => {
                let luv = Luv::from_color_unclamped(xyz);
                let lchuv = Lchuv::from_color_unclamped(luv);
                let c = Hsluv::from_color_unclamped(lchuv);
                (c.hue.into_positive_degrees(), c.saturation, c.l)
            }
            "xyz" => xyz.into_components(),
            "xyz_d50" => {
                let adapted: Xyz<D50, f64> = Xyz::adapt_from_unclamped(xyz);
                adapted.into_components()
            }
            "yxy" => Yxy::from_color_unclamped(xyz).into_components(),
            "luma" => {
                let c = Luma::<palette::encoding::Srgb, f64>::from_color_unclamped(xyz);
                (c.luma, 0.0, 0.0)
            }
            "luma_linear" => {
                let c = Luma::<palette::encoding::Linear<D65>, f64>::from_color_unclamped(xyz);
                (c.luma, 0.0, 0.0)
            }
            "luma_gamma" => {
                let c = Luma::<palette::encoding::Gamma<D65>, f64>::from_color_unclamped(xyz);
                (c.luma, 0.0, 0.0)
            }
            "oklab" => Oklab::from_color_unclamped(xyz).into_components(),
            "oklch" => {
                let lab = Oklab::from_color_unclamped(xyz);
                let c = Oklch::from_color_unclamped(lab);
                (c.l, c.chroma, c.hue.into_positive_degrees())
            }
            "okhsl" => {
                let lab = Oklab::from_color_unclamped(xyz);
                okhsl_first(Okhsl::from_color_unclamped(lab))
            }
            "okhsv" => {
                let lab = Oklab::from_color_unclamped(xyz);
                let c = Okhsv::from_color_unclamped(lab);
                (c.hue.into_positive_degrees(), c.saturation, c.value)
            }
            "okhwb" => {
                let lab = Oklab::from_color_unclamped(xyz);
                let hsv = Okhsv::from_color_unclamped(lab);
                let c = Okhwb::from_color_unclamped(hsv);
                (c.hue.into_positive_degrees(), c.whiteness, c.blackness)
            }
            "cam16_jch" => {
                partial_components(Cam16Jch::from_xyz(xyz, Self::cam16_parameters().bake()))
            }
            "cam16_jmh" => {
                partial_components(Cam16Jmh::from_xyz(xyz, Self::cam16_parameters().bake()))
            }
            "cam16_jsh" => {
                partial_components(Cam16Jsh::from_xyz(xyz, Self::cam16_parameters().bake()))
            }
            "cam16_qch" => {
                partial_components(Cam16Qch::from_xyz(xyz, Self::cam16_parameters().bake()))
            }
            "cam16_qmh" => {
                partial_components(Cam16Qmh::from_xyz(xyz, Self::cam16_parameters().bake()))
            }
            "cam16_qsh" => {
                partial_components(Cam16Qsh::from_xyz(xyz, Self::cam16_parameters().bake()))
            }
            "cam16_ucs_jab" => {
                let jmh = Cam16Jmh::from_xyz(xyz, Self::cam16_parameters().bake());
                Cam16UcsJab::from_color_unclamped(jmh).into_components()
            }
            "cam16_ucs_jmh" => {
                let jmh = Cam16Jmh::from_xyz(xyz, Self::cam16_parameters().bake());
                let c = Cam16UcsJmh::from_color_unclamped(jmh);
                (c.lightness, c.colorfulness, c.hue.into_positive_degrees())
            }
            "lms" => BradfordLms::<D65, f64>::from_color_unclamped(xyz).into_components(),
            "lms_von_kries" => VonKriesLms::<D65, f64>::from_color_unclamped(xyz).into_components(),
            _ => unreachable!("canonical_space returned an unknown space"),
        };
        Ok(Self::new(space, c1, c2, c3, alpha))
    }

    /// 转换到目标空间，不裁剪 gamut（palette 的 `FromColorUnclamped` 语义）。
    ///
    /// Convert without gamut clamping (`FromColorUnclamped` semantics).
    pub fn convert(&self, fn_name: &str, target: &str) -> Result<Self, ExtensionError> {
        let target = space_arg(fn_name, target)?;
        if self.require_space(fn_name)? == target {
            return Ok(self.clone());
        }
        let xyz = self.to_xyz()?;
        Self::from_xyz(fn_name, target, xyz, self.alpha)
    }

    /// 与 `convert` 相同，但结果按目标空间的边界裁剪（palette 的 `Clamp`）。
    ///
    /// Same as `convert`, then clamp to the target space's bounds.
    pub fn convert_clamped(&self, fn_name: &str, target: &str) -> Result<Self, ExtensionError> {
        let converted = self.convert(fn_name, target)?;
        converted.clamped(fn_name)
    }

    /// 用 palette 的 `Clamp` 把各分量收进标准范围。
    ///
    /// Clamp the components to the target space's standard range.
    pub fn clamped(&self, fn_name: &str) -> Result<Self, ExtensionError> {
        use palette::Clamp;
        let (c1, c2, c3) = match self.any_as(fn_name)? {
            AnyColor::Srgb(c) => c.clamp().into_components(),
            AnyColor::SrgbLinear(c) => c.clamp().into_components(),
            AnyColor::SrgbGamma(c) => c.clamp().into_components(),
            AnyColor::AdobeRgb(c) => c.clamp().into_components(),
            AnyColor::AdobeRgbLinear(c) => c.clamp().into_components(),
            AnyColor::DisplayP3(c) => c.clamp().into_components(),
            AnyColor::DisplayP3Linear(c) => c.clamp().into_components(),
            AnyColor::DciP3(c) => c.clamp().into_components(),
            AnyColor::DciP3Linear(c) => c.clamp().into_components(),
            AnyColor::ProPhoto(c) => c.clamp().into_components(),
            AnyColor::ProPhotoLinear(c) => c.clamp().into_components(),
            AnyColor::Rec709(c) => c.clamp().into_components(),
            AnyColor::Rec2020(c) => c.clamp().into_components(),
            AnyColor::Rec2020Linear(c) => c.clamp().into_components(),
            AnyColor::Hsl(c) => hue_first(c.clamp()),
            AnyColor::Hsv(c) => {
                let c = c.clamp();
                (c.hue.into_positive_degrees(), c.saturation, c.value)
            }
            AnyColor::Hwb(c) => {
                let c = c.clamp();
                (c.hue.into_positive_degrees(), c.whiteness, c.blackness)
            }
            AnyColor::Lab(c) => c.clamp().into_components(),
            AnyColor::Lch(c) => {
                let c = c.clamp();
                (c.l, c.chroma, c.hue.into_positive_degrees())
            }
            AnyColor::Luv(c) => c.clamp().into_components(),
            AnyColor::Lchuv(c) => {
                let c = c.clamp();
                (c.l, c.chroma, c.hue.into_positive_degrees())
            }
            AnyColor::Hsluv(c) => {
                let c = c.clamp();
                (c.hue.into_positive_degrees(), c.saturation, c.l)
            }
            AnyColor::Xyz(c) => c.clamp().into_components(),
            AnyColor::XyzD50(c) => c.clamp().into_components(),
            AnyColor::Yxy(c) => c.clamp().into_components(),
            AnyColor::Luma(c) => (c.clamp().luma, 0.0, 0.0),
            AnyColor::LumaLinear(c) => (c.clamp().luma, 0.0, 0.0),
            AnyColor::LumaGamma(c) => (c.clamp().luma, 0.0, 0.0),
            AnyColor::Oklab(c) => c.clamp().into_components(),
            AnyColor::Oklch(c) => {
                let c = c.clamp();
                (c.l, c.chroma, c.hue.into_positive_degrees())
            }
            AnyColor::Okhsl(c) => okhsl_first(c.clamp()),
            AnyColor::Okhsv(c) => {
                let c = c.clamp();
                (c.hue.into_positive_degrees(), c.saturation, c.value)
            }
            AnyColor::Okhwb(c) => {
                let c = c.clamp();
                (c.hue.into_positive_degrees(), c.whiteness, c.blackness)
            }
            AnyColor::Cam16Jch(c) => partial_components(c.clamp()),
            AnyColor::Cam16Jmh(c) => partial_components(c.clamp()),
            AnyColor::Cam16Jsh(c) => partial_components(c.clamp()),
            AnyColor::Cam16Qch(c) => partial_components(c.clamp()),
            AnyColor::Cam16Qmh(c) => partial_components(c.clamp()),
            AnyColor::Cam16Qsh(c) => partial_components(c.clamp()),
            AnyColor::Cam16UcsJab(c) => c.clamp().into_components(),
            AnyColor::Cam16UcsJmh(c) => {
                let c = c.clamp();
                (c.lightness, c.colorfulness, c.hue.into_positive_degrees())
            }
            AnyColor::Lms(c) => c.clamp().into_components(),
            AnyColor::LmsVonKries(c) => c.clamp().into_components(),
        };
        Ok(Self::new(&self.space, c1, c2, c3, self.alpha))
    }

    /// 是否所有分量都在标准范围内（palette 的 `Clamp` 的伴随判定）。
    ///
    /// Whether every component is already within the space's bounds.
    pub fn is_within_bounds(&self, fn_name: &str) -> Result<bool, ExtensionError> {
        use palette::IsWithinBounds;
        let inside = match self.any_as(fn_name)? {
            AnyColor::Srgb(c) => c.is_within_bounds(),
            AnyColor::SrgbLinear(c) => c.is_within_bounds(),
            AnyColor::SrgbGamma(c) => c.is_within_bounds(),
            AnyColor::AdobeRgb(c) => c.is_within_bounds(),
            AnyColor::AdobeRgbLinear(c) => c.is_within_bounds(),
            AnyColor::DisplayP3(c) => c.is_within_bounds(),
            AnyColor::DisplayP3Linear(c) => c.is_within_bounds(),
            AnyColor::DciP3(c) => c.is_within_bounds(),
            AnyColor::DciP3Linear(c) => c.is_within_bounds(),
            AnyColor::ProPhoto(c) => c.is_within_bounds(),
            AnyColor::ProPhotoLinear(c) => c.is_within_bounds(),
            AnyColor::Rec709(c) => c.is_within_bounds(),
            AnyColor::Rec2020(c) => c.is_within_bounds(),
            AnyColor::Rec2020Linear(c) => c.is_within_bounds(),
            AnyColor::Hsl(c) => c.is_within_bounds(),
            AnyColor::Hsv(c) => c.is_within_bounds(),
            AnyColor::Hwb(c) => c.is_within_bounds(),
            AnyColor::Lab(c) => c.is_within_bounds(),
            AnyColor::Lch(c) => c.is_within_bounds(),
            AnyColor::Luv(c) => c.is_within_bounds(),
            AnyColor::Lchuv(c) => c.is_within_bounds(),
            AnyColor::Hsluv(c) => c.is_within_bounds(),
            AnyColor::Xyz(c) => c.is_within_bounds(),
            AnyColor::XyzD50(c) => c.is_within_bounds(),
            AnyColor::Yxy(c) => c.is_within_bounds(),
            AnyColor::Luma(c) => c.is_within_bounds(),
            AnyColor::LumaLinear(c) => c.is_within_bounds(),
            AnyColor::LumaGamma(c) => c.is_within_bounds(),
            AnyColor::Oklab(c) => c.is_within_bounds(),
            AnyColor::Oklch(c) => c.is_within_bounds(),
            AnyColor::Okhsl(c) => c.is_within_bounds(),
            AnyColor::Okhsv(c) => c.is_within_bounds(),
            AnyColor::Okhwb(c) => c.is_within_bounds(),
            AnyColor::Cam16Jch(c) => c.is_within_bounds(),
            AnyColor::Cam16Jmh(c) => c.is_within_bounds(),
            AnyColor::Cam16Jsh(c) => c.is_within_bounds(),
            AnyColor::Cam16Qch(c) => c.is_within_bounds(),
            AnyColor::Cam16Qmh(c) => c.is_within_bounds(),
            AnyColor::Cam16Qsh(c) => c.is_within_bounds(),
            AnyColor::Cam16UcsJab(c) => c.is_within_bounds(),
            AnyColor::Cam16UcsJmh(c) => c.is_within_bounds(),
            AnyColor::Lms(c) => c.is_within_bounds(),
            AnyColor::LmsVonKries(c) => c.is_within_bounds(),
        };
        Ok(inside)
    }

    /// 保留 alpha，替换空间与三分量。
    ///
    /// Replace space and components, keeping alpha.
    pub fn with_components(&self, space: &str, c1: f64, c2: f64, c3: f64) -> Self {
        Self::new(space, c1, c2, c3, self.alpha)
    }

    /// alpha 是否在 [0, 1] 内；premultiply / unpremultiply 需要一个可逆的 alpha。
    ///
    /// `Err` when alpha is outside [0, 1]; premultiply / unpremultiply need it invertible.
    pub fn require_alpha01(&self, fn_name: &str) -> Result<f64, ExtensionError> {
        if !(0.0..=1.0).contains(&self.alpha) {
            return Err(duck_error(format!(
                "{fn_name}: alpha must be within [0, 1], got {}",
                self.alpha
            )));
        }
        Ok(self.alpha)
    }

    /// 两个颜色是否渲染成同一份 8 位 sRGB（alpha 按两位小数比较），与所在空间无关。
    ///
    /// Whether both colors render as the same 8-bit sRGB value, comparing alpha at two decimal
    /// places. The spaces may differ.
    pub fn same_rgba8(&self, other: &Self, fn_name: &str) -> Result<bool, ExtensionError> {
        let a = self.convert(fn_name, "srgb")?;
        let b = other.convert(fn_name, "srgb")?;
        let a8: Srgb<u8> = Srgb::<f64>::new(a.c1, a.c2, a.c3).into_format();
        let b8: Srgb<u8> = Srgb::<f64>::new(b.c1, b.c2, b.c3).into_format();
        let alpha = |value: f64| (value.clamp(0.0, 1.0) * 100.0).round() as i64;
        Ok(a8 == b8 && alpha(a.alpha) == alpha(b.alpha))
    }
}

/// `Hsl` 的 hue 是首个分量，摊成 `(hue°, saturation, lightness)`。
///
/// `Hsl` is hue-first: `(hue°, saturation, lightness)`.
fn hue_first(c: Hsl<palette::encoding::Srgb, f64>) -> (f64, f64, f64) {
    (c.hue.into_positive_degrees(), c.saturation, c.lightness)
}

/// `Okhsl` 的 hue 也是首个分量。
///
/// `Okhsl` is also hue-first.
fn okhsl_first(c: Okhsl<f64>) -> (f64, f64, f64) {
    (c.hue.into_positive_degrees(), c.saturation, c.lightness)
}

/// CAM16 部分类型：`(luminance/brightness, chromaticity, hue°)`。
///
/// A partial CAM16 value as `(luminance/brightness, chromaticity, hue°)`.
fn partial_components<C: Cam16PartialComponents>(c: C) -> (f64, f64, f64) {
    c.into_triple()
}

/// 给六个 CAM16 部分类型提供统一的分量摊平入口。
///
/// Uniform component flattening for the six partial CAM16 types.
trait Cam16PartialComponents {
    fn into_triple(self) -> (f64, f64, f64);
}

macro_rules! cam16_partial_triple {
    ($($ty:ident $luminance:ident $chromaticity:ident),+ $(,)?) => {
        $(
            impl Cam16PartialComponents for $ty<f64> {
                fn into_triple(self) -> (f64, f64, f64) {
                    (
                        self.$luminance,
                        self.$chromaticity,
                        self.hue.into_positive_degrees(),
                    )
                }
            }
        )+
    };
}

cam16_partial_triple! {
    Cam16Jch lightness chroma,
    Cam16Jmh lightness colorfulness,
    Cam16Jsh lightness saturation,
    Cam16Qch brightness chroma,
    Cam16Qmh brightness colorfulness,
    Cam16Qsh brightness saturation,
}

/// `Cam16` 完整属性（palette 的 `Cam16` 有 5 个相关属性 + hue，SQL 结构体装不下三分量）。
///
/// The full set of CAM16 attributes; palette's `Cam16` has five correlated attributes plus hue,
/// more than the three component slots of `plt_color`.
#[derive(Debug, Clone, Default, DuckStruct)]
#[duck(sql_name = "plt_cam16_full", create_type = true)]
pub struct PltCam16Full {
    pub lightness: f64,
    pub chroma: f64,
    pub hue: f64,
    pub brightness: f64,
    pub colorfulness: f64,
    pub saturation: f64,
    pub alpha: f64,
}

impl From<Cam16<f64>> for PltCam16Full {
    fn from(c: Cam16<f64>) -> Self {
        Self {
            lightness: c.lightness,
            chroma: c.chroma,
            hue: c.hue.into_positive_degrees(),
            brightness: c.brightness,
            colorfulness: c.colorfulness,
            saturation: c.saturation,
            alpha: 1.0,
        }
    }
}

impl PltCam16Full {
    /// 按 palette 的结构体字面量重建 `Cam16`。palette 假定这几个属性彼此自洽，
    /// 它只用其中一部分去反算 XYZ；要精确控制用哪些属性，请改用六个 `cam16_*` 部分类型。
    ///
    /// Rebuild `Cam16` from the struct fields. palette assumes the correlated attributes are
    /// consistent and uses only some of them when converting back; use the partial types for
    /// precise control.
    pub fn to_cam16(&self) -> Cam16<f64> {
        Cam16 {
            lightness: self.lightness,
            chroma: self.chroma,
            hue: palette::hues::Cam16Hue::from_degrees(self.hue),
            brightness: self.brightness,
            colorfulness: self.colorfulness,
            saturation: self.saturation,
        }
    }
}

/// 把函数参数里的空间名规范化后返回；错误信息带函数名。
///
/// Normalize a function argument's space name, with the function name in the error.
pub fn space_arg(fn_name: &str, space: &str) -> Result<&'static str, ExtensionError> {
    canonical_space(space).ok_or_else(|| {
        duck_error(format!(
            "{fn_name}: unknown color space '{space}'; see plt_color_spaces()"
        ))
    })
}

/// 包装调用的统一错误构造（保持函数名前缀一致）。
///
/// Uniform error builder that keeps the function-name prefix.
pub fn bad_arg(fn_name: &str, message: impl std::fmt::Display) -> ExtensionError {
    err(format!("{fn_name}: {message}"))
}
