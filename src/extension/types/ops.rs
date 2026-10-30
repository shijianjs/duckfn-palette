// ============================================================================
// 逐空间操作：mix / lighten / darken / saturate / desaturate / hue / premultiply。
//
// palette 的这些 trait 都是「按空间定义」的（HSL 的 lightness 与 Oklch 的 lightness 不是一件事），
// 所以这里不做隐式换空间：操作在颜色自己所在的空间里执行，空间不支持该 trait 时报错，并在消息里
// 建议先 `plt_convert` 到一个支持它的空间。`plt_mix` 是唯一的例外 —— 两个操作数必须同处一空间，
// 第二个颜色会被自动转到第一个颜色的空间。
//
// `SqlComponents` 把 `plt_color` 的 (c1, c2, c3) 与各 palette 类型互转，分量顺序与该空间构造函数
// 一致（hue 在前的 Hsl/Hsv/Hwb/Hsluv/Okh*，hue 在后的 Lch/Lchuv/Oklch，CAM16 部分类型是
// (亮度, 彩度, hue)）。每个操作族有一份自己的「空间名 → 类型」清单，取自 palette 的 impl 清单。
//
// Per-space operations. palette defines these traits per color space, so nothing is converted
// implicitly: an operation runs in the color's own space and fails with a "convert first" hint
// when the space lacks the trait. `plt_mix` is the exception and converts the second operand into
// the first one's space. `SqlComponents` bridges the SQL triple and each palette type in the
// constructor's component order; each operation family owns its name → type list, taken from
// palette's own impl lists.
// ============================================================================

// 同 color.rs：`srgb_gamma` / `luma_gamma` 用的是 palette 标为 deprecated 的 gamma 2.2 类型。
#![allow(deprecated)]

use palette::blend::{PreAlpha, Premultiply};
use palette::cam16::{
    Cam16Jch, Cam16Jmh, Cam16Jsh, Cam16Qch, Cam16Qmh, Cam16Qsh, Cam16UcsJab, Cam16UcsJmh,
};
use palette::encoding;
use palette::lms::{BradfordLms, Lms, VonKriesLms};
use palette::luma::Luma;
use palette::rgb::{
    AdobeRgb, DciP3, DisplayP3, GammaSrgb, LinAdobeRgb, LinDciP3, LinDisplayP3, LinProPhotoRgb,
    LinRec2020, LinSrgb, ProPhotoRgb, Rec709, Rec2020, Rgb, Srgb,
};
use palette::white_point::{D50, D65};
use palette::{
    Darken, Desaturate, GetHue, Hsl, Hsluv, Hsv, Hwb, Lab, Lch, Lchuv, Lighten, Luv, Mix, Okhsl,
    Okhsv, Okhwb, Oklab, Oklch, Saturate, ShiftHue, WithHue, Xyz, Yxy,
};
use quack_rs::error::ExtensionError;

use super::color::{PltColor, bad_arg};

/// `plt_color` 的三分量与 palette 类型互转；分量顺序与 palette 的 `new` 一致。
///
/// Convert between a `plt_color` triple and a palette type, in the constructor's component order.
pub trait SqlComponents: Sized + Copy {
    fn from_triple(t: (f64, f64, f64)) -> Self;
    fn to_triple(self) -> (f64, f64, f64);
}

/// hue 在前的三元组：`(hue°, second, third)`，hue 归一化到 0..360。
///
/// Hue-first triple: `(hue°, second, third)`, with the hue normalized to 0..360.
fn hue_first_triple(hue: f64, second: f64, third: f64) -> (f64, f64, f64) {
    (hue.rem_euclid(360.0), second, third)
}

/// hue 在后的三元组：`(first, second, hue°)`。
///
/// Hue-last triple: `(first, second, hue°)`.
fn hue_last_triple(first: f64, second: f64, hue: f64) -> (f64, f64, f64) {
    (first, second, hue.rem_euclid(360.0))
}

macro_rules! components3 {
    ($ty:ident, [$($gen:ident),*]; $($field:ident),+) => {
        impl<$($gen,)*> SqlComponents for $ty<$($gen,)* f64> {
            fn from_triple(t: (f64, f64, f64)) -> Self {
                Self::new(t.0, t.1, t.2)
            }

            fn to_triple(self) -> (f64, f64, f64) {
                ($(self.$field,)+)
            }
        }
    };
}

macro_rules! components_hue_first {
    ($ty:ident, [$($gen:ident),*]; $second:ident, $third:ident) => {
        impl<$($gen,)*> SqlComponents for $ty<$($gen,)* f64> {
            fn from_triple(t: (f64, f64, f64)) -> Self {
                Self::new(t.0, t.1, t.2)
            }

            fn to_triple(self) -> (f64, f64, f64) {
                hue_first_triple(self.hue.into_positive_degrees(), self.$second, self.$third)
            }
        }
    };
}

macro_rules! components_hue_last {
    ($ty:ident, [$($gen:ident),*]; $first:ident, $second:ident) => {
        impl<$($gen,)*> SqlComponents for $ty<$($gen,)* f64> {
            fn from_triple(t: (f64, f64, f64)) -> Self {
                Self::new(t.0, t.1, t.2)
            }

            fn to_triple(self) -> (f64, f64, f64) {
                hue_last_triple(self.$first, self.$second, self.hue.into_positive_degrees())
            }
        }
    };
}

components3!(Rgb, [S]; red, green, blue);
components3!(Lab, [Wp]; l, a, b);
components3!(Luv, [Wp]; l, u, v);
components3!(Xyz, [Wp]; x, y, z);
components3!(Yxy, [Wp]; x, y, luma);
components3!(Oklab, []; l, a, b);
components3!(Lms, [M]; long, medium, short);
components3!(Cam16UcsJab, []; lightness, a, b);

components_hue_first!(Hsl, [S]; saturation, lightness);
components_hue_first!(Hsv, [S]; saturation, value);
components_hue_first!(Hwb, [S]; whiteness, blackness);
components_hue_first!(Hsluv, [Wp]; saturation, l);
components_hue_first!(Okhsl, []; saturation, lightness);
components_hue_first!(Okhsv, []; saturation, value);
components_hue_first!(Okhwb, []; whiteness, blackness);

// CAM16 部分类型的构造参数是 `(luminance/brightness, chromaticity, hue)`，hue 在最后。
components_hue_last!(Cam16Jch, []; lightness, chroma);
components_hue_last!(Cam16Jmh, []; lightness, colorfulness);
components_hue_last!(Cam16Jsh, []; lightness, saturation);
components_hue_last!(Cam16Qch, []; brightness, chroma);
components_hue_last!(Cam16Qmh, []; brightness, colorfulness);
components_hue_last!(Cam16Qsh, []; brightness, saturation);
components_hue_last!(Cam16UcsJmh, []; lightness, colorfulness);

components_hue_last!(Lch, [Wp]; l, chroma);
components_hue_last!(Lchuv, [Wp]; l, chroma);
components_hue_last!(Oklch, []; l, chroma);

impl<S> SqlComponents for Luma<S, f64> {
    fn from_triple(t: (f64, f64, f64)) -> Self {
        Self::new(t.0)
    }

    fn to_triple(self) -> (f64, f64, f64) {
        (self.luma, 0.0, 0.0)
    }
}

/// 空间不支持该操作时的统一错误：以函数名开头，并给出下一步。
///
/// Uniform "space does not support this operation" error, starting with the function name.
fn unsupported(fn_name: &str, space: &str) -> ExtensionError {
    bad_arg(
        fn_name,
        format!(
            "color space '{space}' does not support this operation; \
             convert it first, e.g. plt_convert(color, 'oklch')"
        ),
    )
}

/// 一元 trait 方法：`color.$method($arg)`，结果摊回三元组。
///
/// Unary trait method: `color.$method($arg)`, flattened back into a triple.
macro_rules! unary_op {
    ($space:expr, $fn_name:expr, $a:expr, $method:ident, $arg:expr,
     [$($name:literal => $ty:ty),+ $(,)?]) => {{
        let a = $a;
        match $space {
            $(
                $name => {
                    let c = <$ty as SqlComponents>::from_triple(a);
                    Ok(<$ty as SqlComponents>::to_triple(c.$method($arg)))
                }
            )+
            other => Err(unsupported($fn_name, other)),
        }
    }};
}

/// 二元 trait 方法：`a.$method(b, $arg)`。
///
/// Binary trait method: `a.$method(b, $arg)`.
macro_rules! binary_op {
    ($space:expr, $fn_name:expr, $a:expr, $b:expr, $method:ident, $arg:expr,
     [$($name:literal => $ty:ty),+ $(,)?]) => {{
        let (a, b) = ($a, $b);
        match $space {
            $(
                $name => {
                    let lhs = <$ty as SqlComponents>::from_triple(a);
                    let rhs = <$ty as SqlComponents>::from_triple(b);
                    Ok(<$ty as SqlComponents>::to_triple(lhs.$method(rhs, $arg)))
                }
            )+
            other => Err(unsupported($fn_name, other)),
        }
    }};
}

/// `Lighten` / `Darken` 支持的空间（palette 的 `impl_lighten!` 清单）。
///
/// Spaces that implement `Lighten` (and therefore `Darken`).
macro_rules! lighten_spaces {
    ($space:expr, $fn_name:expr, $a:expr, $method:ident, $arg:expr) => {
        unary_op!($space, $fn_name, $a, $method, $arg, [
            "srgb" => Srgb<f64>,
            "srgb_linear" => LinSrgb<f64>,
            "srgb_gamma" => GammaSrgb<f64>,
            "adobe_rgb" => AdobeRgb<f64>,
            "adobe_rgb_linear" => LinAdobeRgb<f64>,
            "display_p3" => DisplayP3<f64>,
            "display_p3_linear" => LinDisplayP3<f64>,
            "dci_p3" => DciP3<f64>,
            "dci_p3_linear" => LinDciP3<f64>,
            "prophoto_rgb" => ProPhotoRgb<f64>,
            "prophoto_rgb_linear" => LinProPhotoRgb<f64>,
            "rec709" => Rec709<f64>,
            "rec709_linear" => LinSrgb<f64>,
            "rec2020" => Rec2020<f64>,
            "rec2020_linear" => LinRec2020<f64>,
            "hsl" => Hsl<encoding::Srgb, f64>,
            "hsv" => Hsv<encoding::Srgb, f64>,
            "hwb" => Hwb<encoding::Srgb, f64>,
            "hsluv" => Hsluv<D65, f64>,
            "lab" => Lab<D65, f64>,
            "lch" => Lch<D65, f64>,
            "luv" => Luv<D65, f64>,
            "lchuv" => Lchuv<D65, f64>,
            "xyz" => Xyz<D65, f64>,
            "xyz_d50" => Xyz<D50, f64>,
            "yxy" => Yxy<D65, f64>,
            "luma" => Luma<encoding::Srgb, f64>,
            "luma_linear" => Luma<encoding::Linear<D65>, f64>,
            "luma_gamma" => Luma<encoding::Gamma<D65>, f64>,
            "oklab" => Oklab<f64>,
            "oklch" => Oklch<f64>,
            "okhsl" => Okhsl<f64>,
            "okhsv" => Okhsv<f64>,
            "okhwb" => Okhwb<f64>,
            "cam16_ucs_jab" => Cam16UcsJab<f64>,
            "cam16_ucs_jmh" => Cam16UcsJmh<f64>,
        ])
    };
}

/// `Saturate` / `Desaturate` 支持的空间（palette 的 `impl_saturate!` 清单）。
///
/// Spaces that implement `Saturate` (and therefore `Desaturate`).
macro_rules! saturate_spaces {
    ($space:expr, $fn_name:expr, $a:expr, $method:ident, $arg:expr) => {
        unary_op!($space, $fn_name, $a, $method, $arg, [
            "hsl" => Hsl<encoding::Srgb, f64>,
            "hsv" => Hsv<encoding::Srgb, f64>,
            "hsluv" => Hsluv<D65, f64>,
            "lch" => Lch<D65, f64>,
            "lchuv" => Lchuv<D65, f64>,
            "okhsl" => Okhsl<f64>,
            "okhsv" => Okhsv<f64>,
            "cam16_ucs_jmh" => Cam16UcsJmh<f64>,
        ])
    };
}

/// `ShiftHue` / `WithHue` 支持的空间（palette 的 `impl_hue_ops!` 清单）。
///
/// Spaces that implement `ShiftHue` and `WithHue`.
macro_rules! hue_spaces {
    ($space:expr, $fn_name:expr, $a:expr, $method:ident, $arg:expr) => {
        unary_op!($space, $fn_name, $a, $method, $arg, [
            "hsl" => Hsl<encoding::Srgb, f64>,
            "hsv" => Hsv<encoding::Srgb, f64>,
            "hwb" => Hwb<encoding::Srgb, f64>,
            "hsluv" => Hsluv<D65, f64>,
            "lch" => Lch<D65, f64>,
            "lchuv" => Lchuv<D65, f64>,
            "oklch" => Oklch<f64>,
            "okhsl" => Okhsl<f64>,
            "okhsv" => Okhsv<f64>,
            "okhwb" => Okhwb<f64>,
            "cam16_jch" => Cam16Jch<f64>,
            "cam16_jmh" => Cam16Jmh<f64>,
            "cam16_jsh" => Cam16Jsh<f64>,
            "cam16_qch" => Cam16Qch<f64>,
            "cam16_qmh" => Cam16Qmh<f64>,
            "cam16_qsh" => Cam16Qsh<f64>,
            "cam16_ucs_jmh" => Cam16UcsJmh<f64>,
        ])
    };
}

/// 能读出 hue 的空间：hue 空间 + 笛卡尔 a/b 或 u/v 的 Lab/Luv/Oklab + 所有 RGB。
///
/// Spaces with a readable hue: the hue spaces plus Cartesian Lab/Luv/Oklab and every RGB.
macro_rules! hue_value_spaces {
    ($space:expr, $fn_name:expr, $a:expr) => {{
        let a = $a;
        match $space {
            "srgb" => Ok(Srgb::<f64>::from_triple(a)
                .get_hue()
                .into_positive_degrees()),
            "srgb_linear" => Ok(LinSrgb::<f64>::from_triple(a)
                .get_hue()
                .into_positive_degrees()),
            "srgb_gamma" => Ok(GammaSrgb::<f64>::from_triple(a)
                .get_hue()
                .into_positive_degrees()),
            "adobe_rgb" => Ok(AdobeRgb::<f64>::from_triple(a)
                .get_hue()
                .into_positive_degrees()),
            "adobe_rgb_linear" => Ok(LinAdobeRgb::<f64>::from_triple(a)
                .get_hue()
                .into_positive_degrees()),
            "display_p3" => Ok(DisplayP3::<f64>::from_triple(a)
                .get_hue()
                .into_positive_degrees()),
            "display_p3_linear" => Ok(LinDisplayP3::<f64>::from_triple(a)
                .get_hue()
                .into_positive_degrees()),
            "dci_p3" => Ok(DciP3::<f64>::from_triple(a)
                .get_hue()
                .into_positive_degrees()),
            "dci_p3_linear" => Ok(LinDciP3::<f64>::from_triple(a)
                .get_hue()
                .into_positive_degrees()),
            "prophoto_rgb" => Ok(ProPhotoRgb::<f64>::from_triple(a)
                .get_hue()
                .into_positive_degrees()),
            "prophoto_rgb_linear" => Ok(LinProPhotoRgb::<f64>::from_triple(a)
                .get_hue()
                .into_positive_degrees()),
            "rec709" => Ok(Rec709::<f64>::from_triple(a)
                .get_hue()
                .into_positive_degrees()),
            "rec709_linear" => Ok(LinSrgb::<f64>::from_triple(a)
                .get_hue()
                .into_positive_degrees()),
            "rec2020" => Ok(Rec2020::<f64>::from_triple(a)
                .get_hue()
                .into_positive_degrees()),
            "rec2020_linear" => Ok(LinRec2020::<f64>::from_triple(a)
                .get_hue()
                .into_positive_degrees()),
            "hsl" => Ok(Hsl::<encoding::Srgb, f64>::from_triple(a)
                .get_hue()
                .into_positive_degrees()),
            "hsv" => Ok(Hsv::<encoding::Srgb, f64>::from_triple(a)
                .get_hue()
                .into_positive_degrees()),
            "hwb" => Ok(Hwb::<encoding::Srgb, f64>::from_triple(a)
                .get_hue()
                .into_positive_degrees()),
            "hsluv" => Ok(Hsluv::<D65, f64>::from_triple(a)
                .get_hue()
                .into_positive_degrees()),
            "lab" => Ok(Lab::<D65, f64>::from_triple(a)
                .get_hue()
                .into_positive_degrees()),
            "lch" => Ok(Lch::<D65, f64>::from_triple(a)
                .get_hue()
                .into_positive_degrees()),
            "luv" => Ok(Luv::<D65, f64>::from_triple(a)
                .get_hue()
                .into_positive_degrees()),
            "lchuv" => Ok(Lchuv::<D65, f64>::from_triple(a)
                .get_hue()
                .into_positive_degrees()),
            "oklab" => Ok(Oklab::<f64>::from_triple(a)
                .get_hue()
                .into_positive_degrees()),
            "oklch" => Ok(Oklch::<f64>::from_triple(a)
                .get_hue()
                .into_positive_degrees()),
            "okhsl" => Ok(Okhsl::<f64>::from_triple(a)
                .get_hue()
                .into_positive_degrees()),
            "okhsv" => Ok(Okhsv::<f64>::from_triple(a)
                .get_hue()
                .into_positive_degrees()),
            "okhwb" => Ok(Okhwb::<f64>::from_triple(a)
                .get_hue()
                .into_positive_degrees()),
            "cam16_jch" => Ok(Cam16Jch::<f64>::from_triple(a)
                .get_hue()
                .into_positive_degrees()),
            "cam16_jmh" => Ok(Cam16Jmh::<f64>::from_triple(a)
                .get_hue()
                .into_positive_degrees()),
            "cam16_jsh" => Ok(Cam16Jsh::<f64>::from_triple(a)
                .get_hue()
                .into_positive_degrees()),
            "cam16_qch" => Ok(Cam16Qch::<f64>::from_triple(a)
                .get_hue()
                .into_positive_degrees()),
            "cam16_qmh" => Ok(Cam16Qmh::<f64>::from_triple(a)
                .get_hue()
                .into_positive_degrees()),
            "cam16_qsh" => Ok(Cam16Qsh::<f64>::from_triple(a)
                .get_hue()
                .into_positive_degrees()),
            "cam16_ucs_jmh" => Ok(Cam16UcsJmh::<f64>::from_triple(a)
                .get_hue()
                .into_positive_degrees()),
            other => Err(unsupported($fn_name, other)),
        }
    }};
}

/// `Premultiply` 支持的空间（palette 的 `impl_premultiply!` 清单）。
///
/// Spaces that implement `Premultiply`.
macro_rules! premultiply_spaces {
    ($space:expr, $fn_name:expr, $a:expr, $alpha:expr, $forward:expr) => {{
        let a = $a;
        let alpha = $alpha;
        let forward = $forward;
        match $space {
            "srgb" => Ok(premultiply_one(Srgb::<f64>::from_triple(a), alpha, forward)),
            "srgb_linear" => Ok(premultiply_one(
                LinSrgb::<f64>::from_triple(a),
                alpha,
                forward,
            )),
            "srgb_gamma" => Ok(premultiply_one(
                GammaSrgb::<f64>::from_triple(a),
                alpha,
                forward,
            )),
            "adobe_rgb" => Ok(premultiply_one(
                AdobeRgb::<f64>::from_triple(a),
                alpha,
                forward,
            )),
            "adobe_rgb_linear" => Ok(premultiply_one(
                LinAdobeRgb::<f64>::from_triple(a),
                alpha,
                forward,
            )),
            "display_p3" => Ok(premultiply_one(
                DisplayP3::<f64>::from_triple(a),
                alpha,
                forward,
            )),
            "display_p3_linear" => Ok(premultiply_one(
                LinDisplayP3::<f64>::from_triple(a),
                alpha,
                forward,
            )),
            "dci_p3" => Ok(premultiply_one(
                DciP3::<f64>::from_triple(a),
                alpha,
                forward,
            )),
            "dci_p3_linear" => Ok(premultiply_one(
                LinDciP3::<f64>::from_triple(a),
                alpha,
                forward,
            )),
            "prophoto_rgb" => Ok(premultiply_one(
                ProPhotoRgb::<f64>::from_triple(a),
                alpha,
                forward,
            )),
            "prophoto_rgb_linear" => Ok(premultiply_one(
                LinProPhotoRgb::<f64>::from_triple(a),
                alpha,
                forward,
            )),
            "rec709" => Ok(premultiply_one(
                Rec709::<f64>::from_triple(a),
                alpha,
                forward,
            )),
            "rec709_linear" => Ok(premultiply_one(
                LinSrgb::<f64>::from_triple(a),
                alpha,
                forward,
            )),
            "rec2020" => Ok(premultiply_one(
                Rec2020::<f64>::from_triple(a),
                alpha,
                forward,
            )),
            "rec2020_linear" => Ok(premultiply_one(
                LinRec2020::<f64>::from_triple(a),
                alpha,
                forward,
            )),
            "lab" => Ok(premultiply_one(
                Lab::<D65, f64>::from_triple(a),
                alpha,
                forward,
            )),
            "luv" => Ok(premultiply_one(
                Luv::<D65, f64>::from_triple(a),
                alpha,
                forward,
            )),
            "xyz" => Ok(premultiply_one(
                Xyz::<D65, f64>::from_triple(a),
                alpha,
                forward,
            )),
            "xyz_d50" => Ok(premultiply_one(
                Xyz::<D50, f64>::from_triple(a),
                alpha,
                forward,
            )),
            "yxy" => Ok(premultiply_one(
                Yxy::<D65, f64>::from_triple(a),
                alpha,
                forward,
            )),
            "luma" => Ok(premultiply_one(
                Luma::<encoding::Srgb, f64>::from_triple(a),
                alpha,
                forward,
            )),
            "luma_linear" => Ok(premultiply_one(
                Luma::<encoding::Linear<D65>, f64>::from_triple(a),
                alpha,
                forward,
            )),
            "luma_gamma" => Ok(premultiply_one(
                Luma::<encoding::Gamma<D65>, f64>::from_triple(a),
                alpha,
                forward,
            )),
            "oklab" => Ok(premultiply_one(
                Oklab::<f64>::from_triple(a),
                alpha,
                forward,
            )),
            "cam16_ucs_jab" => Ok(premultiply_one(
                Cam16UcsJab::<f64>::from_triple(a),
                alpha,
                forward,
            )),
            "lms" => Ok(premultiply_one(
                BradfordLms::<D65, f64>::from_triple(a),
                alpha,
                forward,
            )),
            "lms_von_kries" => Ok(premultiply_one(
                VonKriesLms::<D65, f64>::from_triple(a),
                alpha,
                forward,
            )),
            other => Err(unsupported($fn_name, other)),
        }
    }};
}

/// 单个颜色的 premultiply / unpremultiply，两种方向共用一个泛型入口。
///
/// One color's premultiply / unpremultiply, both directions through one generic entry.
fn premultiply_one<C>(color: C, alpha: f64, forward: bool) -> (f64, f64, f64)
where
    C: Premultiply<Scalar = f64> + SqlComponents,
{
    let out = if forward {
        color.premultiply(alpha).color
    } else {
        C::unpremultiply(PreAlpha { color, alpha }).0
    };
    out.to_triple()
}

impl PltColor {
    /// 在当前空间里按 `factor` 线性插值；另一个颜色会先转到当前空间，alpha 同步插值。
    ///
    /// Linear interpolation in this color's space; the other color is converted into this space
    /// first, and alpha is interpolated as well.
    pub fn mix_with(
        &self,
        other: &Self,
        factor: f64,
        fn_name: &str,
    ) -> Result<Self, ExtensionError> {
        let space = self.require_space(fn_name)?;
        let other = if other.require_space(fn_name)? == space {
            other.clone()
        } else {
            other.convert(fn_name, space)?
        };
        let (c1, c2, c3) = binary_op!(
            space,
            fn_name,
            (self.c1, self.c2, self.c3),
            (other.c1, other.c2, other.c3),
            mix,
            factor,
            [
                "srgb" => Srgb<f64>,
                "srgb_linear" => LinSrgb<f64>,
                "srgb_gamma" => GammaSrgb<f64>,
                "adobe_rgb" => AdobeRgb<f64>,
                "adobe_rgb_linear" => LinAdobeRgb<f64>,
                "display_p3" => DisplayP3<f64>,
                "display_p3_linear" => LinDisplayP3<f64>,
                "dci_p3" => DciP3<f64>,
                "dci_p3_linear" => LinDciP3<f64>,
                "prophoto_rgb" => ProPhotoRgb<f64>,
                "prophoto_rgb_linear" => LinProPhotoRgb<f64>,
                "rec709" => Rec709<f64>,
                "rec709_linear" => LinSrgb<f64>,
                "rec2020" => Rec2020<f64>,
                "rec2020_linear" => LinRec2020<f64>,
                "hsl" => Hsl<encoding::Srgb, f64>,
                "hsv" => Hsv<encoding::Srgb, f64>,
                "hwb" => Hwb<encoding::Srgb, f64>,
                "hsluv" => Hsluv<D65, f64>,
                "lab" => Lab<D65, f64>,
                "lch" => Lch<D65, f64>,
                "luv" => Luv<D65, f64>,
                "lchuv" => Lchuv<D65, f64>,
                "xyz" => Xyz<D65, f64>,
                "xyz_d50" => Xyz<D50, f64>,
                "yxy" => Yxy<D65, f64>,
                "luma" => Luma<encoding::Srgb, f64>,
                "luma_linear" => Luma<encoding::Linear<D65>, f64>,
                "luma_gamma" => Luma<encoding::Gamma<D65>, f64>,
                "oklab" => Oklab<f64>,
                "oklch" => Oklch<f64>,
                "okhsl" => Okhsl<f64>,
                "okhsv" => Okhsv<f64>,
                "okhwb" => Okhwb<f64>,
                "cam16_jch" => Cam16Jch<f64>,
                "cam16_jmh" => Cam16Jmh<f64>,
                "cam16_jsh" => Cam16Jsh<f64>,
                "cam16_qch" => Cam16Qch<f64>,
                "cam16_qmh" => Cam16Qmh<f64>,
                "cam16_qsh" => Cam16Qsh<f64>,
                "cam16_ucs_jab" => Cam16UcsJab<f64>,
                "cam16_ucs_jmh" => Cam16UcsJmh<f64>,
                "lms" => BradfordLms<D65, f64>,
                "lms_von_kries" => VonKriesLms<D65, f64>,
            ]
        )?;
        let t = factor.clamp(0.0, 1.0);
        Ok(Self::new(
            space,
            c1,
            c2,
            c3,
            self.alpha + (other.alpha - self.alpha) * t,
        ))
    }

    /// 保留空间与 alpha，只换分量 —— 各操作的收尾。
    ///
    /// Keep space and alpha, replace only the components.
    fn recomponent(&self, space: &str, t: (f64, f64, f64)) -> Self {
        Self::new(space, t.0, t.1, t.2, self.alpha)
    }

    /// 在 sRGB 里朝 `target` 混合（tint/shade 的共用实现），alpha 不变，结果回到原空间。
    ///
    /// Blend towards `target` in sRGB, keeping alpha, and convert back to the input space.
    fn blend_towards(
        &self,
        target: f64,
        amount: f64,
        fn_name: &str,
    ) -> Result<Self, ExtensionError> {
        let space = self.require_space(fn_name)?;
        let srgb = self.convert(fn_name, "srgb")?;
        let amount = amount.clamp(0.0, 1.0);
        let blend = |value: f64| (value * (1.0 - amount) + target * amount).clamp(0.0, 1.0);
        let out = Self::new(
            "srgb",
            blend(srgb.c1),
            blend(srgb.c2),
            blend(srgb.c3),
            self.alpha,
        );
        out.convert(fn_name, space)
    }

    /// 在每个 sRGB 通道上加上 `amount`（0..1 的整段量程），alpha 不变。
    ///
    /// Add `amount` (a fraction of the full 0..1 channel range) to every sRGB channel. This is
    /// tinycolor's `brighten`, expressed in the extension's 0..1 scale.
    pub fn brighten(&self, amount: f64, fn_name: &str) -> Result<Self, ExtensionError> {
        let space = self.require_space(fn_name)?;
        let srgb = self.convert(fn_name, "srgb")?;
        let amount = amount.clamp(0.0, 1.0);
        let add = |value: f64| (value + amount).clamp(0.0, 1.0);
        let out = Self::new("srgb", add(srgb.c1), add(srgb.c2), add(srgb.c3), self.alpha);
        out.convert(fn_name, space)
    }

    /// 转成 HSL、把饱和度降到 0，再回到原空间（灰色的"亮度"取 HSL 的 L）。
    ///
    /// Drop HSL saturation to zero and convert back, so the grey keeps the HSL lightness.
    pub fn greyscale(&self, fn_name: &str) -> Result<Self, ExtensionError> {
        let space = self.require_space(fn_name)?;
        let hsl = self.convert(fn_name, "hsl")?;
        let grey = hsl.desaturate(1.0, fn_name)?;
        grey.convert(fn_name, space)
    }

    /// 朝白色混合 `amount`（0 不变，1 全白），alpha 不变。
    ///
    /// Mix towards white by `amount` (0 keeps the color, 1 makes it white); alpha is unchanged.
    pub fn tint(&self, amount: f64, fn_name: &str) -> Result<Self, ExtensionError> {
        self.blend_towards(1.0, amount, fn_name)
    }

    /// 朝黑色混合 `amount`（0 不变，1 全黑），alpha 不变。
    ///
    /// Mix towards black by `amount` (0 keeps the color, 1 makes it black); alpha is unchanged.
    pub fn shade(&self, amount: f64, fn_name: &str) -> Result<Self, ExtensionError> {
        self.blend_towards(0.0, amount, fn_name)
    }

    /// 单色方案：固定色相与饱和度，让 HSV 的 value 从当前值起按 `1/count` 循环。
    ///
    /// A monochromatic scheme: keep hue and saturation, cycle HSV value by `1/count` from the
    /// current value. This is tinycolor's `monochromatic`; results stay in the input space.
    pub fn monochromatic(&self, count: usize, fn_name: &str) -> Result<Vec<Self>, ExtensionError> {
        let space = self.require_space(fn_name)?;
        if count == 0 {
            return Ok(Vec::new());
        }
        let hsv = self.convert(fn_name, "hsv")?;
        let step = 1.0 / count as f64;
        let mut colors = Vec::with_capacity(count);
        for index in 0..count {
            let value = (hsv.c3 + index as f64 * step).rem_euclid(1.0);
            let color = Self::new("hsv", hsv.c1, hsv.c2, value, self.alpha);
            colors.push(color.convert(fn_name, space)?);
        }
        Ok(colors)
    }

    /// `Lighten::lighten`：按比例朝最大亮度靠拢。
    ///
    /// `Lighten::lighten`: scale towards the maximum lightness.
    pub fn lighten(&self, factor: f64, fn_name: &str) -> Result<Self, ExtensionError> {
        let space = self.require_space(fn_name)?;
        let a = (self.c1, self.c2, self.c3);
        let t = lighten_spaces!(space, fn_name, a, lighten, factor)?;
        Ok(self.recomponent(space, t))
    }

    /// `Lighten::lighten_fixed`：按绝对量提高亮度。
    ///
    /// `Lighten::lighten_fixed`: raise lightness by an absolute amount.
    pub fn lighten_fixed(&self, amount: f64, fn_name: &str) -> Result<Self, ExtensionError> {
        let space = self.require_space(fn_name)?;
        let a = (self.c1, self.c2, self.c3);
        let t = lighten_spaces!(space, fn_name, a, lighten_fixed, amount)?;
        Ok(self.recomponent(space, t))
    }

    /// `Darken::darken`：按比例朝最小亮度靠拢。
    ///
    /// `Darken::darken`: scale towards the minimum lightness.
    pub fn darken(&self, factor: f64, fn_name: &str) -> Result<Self, ExtensionError> {
        let space = self.require_space(fn_name)?;
        let a = (self.c1, self.c2, self.c3);
        let t = lighten_spaces!(space, fn_name, a, darken, factor)?;
        Ok(self.recomponent(space, t))
    }

    /// `Darken::darken_fixed`：按绝对量降低亮度。
    ///
    /// `Darken::darken_fixed`: lower lightness by an absolute amount.
    pub fn darken_fixed(&self, amount: f64, fn_name: &str) -> Result<Self, ExtensionError> {
        let space = self.require_space(fn_name)?;
        let a = (self.c1, self.c2, self.c3);
        let t = lighten_spaces!(space, fn_name, a, darken_fixed, amount)?;
        Ok(self.recomponent(space, t))
    }

    /// `Saturate::saturate`：按比例提高饱和度/彩度。
    ///
    /// `Saturate::saturate`: scale towards maximum saturation or chroma.
    pub fn saturate(&self, factor: f64, fn_name: &str) -> Result<Self, ExtensionError> {
        let space = self.require_space(fn_name)?;
        let a = (self.c1, self.c2, self.c3);
        let t = saturate_spaces!(space, fn_name, a, saturate, factor)?;
        Ok(self.recomponent(space, t))
    }

    /// `Saturate::saturate_fixed`：按绝对量提高饱和度/彩度。
    ///
    /// `Saturate::saturate_fixed`: raise saturation or chroma by an absolute amount.
    pub fn saturate_fixed(&self, amount: f64, fn_name: &str) -> Result<Self, ExtensionError> {
        let space = self.require_space(fn_name)?;
        let a = (self.c1, self.c2, self.c3);
        let t = saturate_spaces!(space, fn_name, a, saturate_fixed, amount)?;
        Ok(self.recomponent(space, t))
    }

    /// `Desaturate::desaturate`：按比例朝最小饱和度靠拢。
    ///
    /// `Desaturate::desaturate`: scale towards minimum saturation or chroma.
    pub fn desaturate(&self, factor: f64, fn_name: &str) -> Result<Self, ExtensionError> {
        let space = self.require_space(fn_name)?;
        let a = (self.c1, self.c2, self.c3);
        let t = saturate_spaces!(space, fn_name, a, desaturate, factor)?;
        Ok(self.recomponent(space, t))
    }

    /// `Desaturate::desaturate_fixed`：按绝对量降低饱和度/彩度。
    ///
    /// `Desaturate::desaturate_fixed`: lower saturation or chroma by an absolute amount.
    pub fn desaturate_fixed(&self, amount: f64, fn_name: &str) -> Result<Self, ExtensionError> {
        let space = self.require_space(fn_name)?;
        let a = (self.c1, self.c2, self.c3);
        let t = saturate_spaces!(space, fn_name, a, desaturate_fixed, amount)?;
        Ok(self.recomponent(space, t))
    }

    /// `ShiftHue::shift_hue`：色相位移（度）。
    ///
    /// `ShiftHue::shift_hue`: rotate the hue by degrees.
    pub fn shift_hue(&self, amount: f64, fn_name: &str) -> Result<Self, ExtensionError> {
        let space = self.require_space(fn_name)?;
        let a = (self.c1, self.c2, self.c3);
        let t = hue_spaces!(space, fn_name, a, shift_hue, amount)?;
        Ok(self.recomponent(space, t))
    }

    /// `WithHue::with_hue`：把色相设为给定值（度）。
    ///
    /// `WithHue::with_hue`: set the hue to the given degrees.
    pub fn with_hue(&self, hue: f64, fn_name: &str) -> Result<Self, ExtensionError> {
        let space = self.require_space(fn_name)?;
        let a = (self.c1, self.c2, self.c3);
        let t = hue_spaces!(space, fn_name, a, with_hue, hue)?;
        Ok(self.recomponent(space, t))
    }

    /// `GetHue::get_hue`：色相角（度，0..360）；灰阶返回 0。
    ///
    /// `GetHue::get_hue`: hue in degrees (0..360); gray returns 0.
    pub fn get_hue(&self, fn_name: &str) -> Result<f64, ExtensionError> {
        let space = self.require_space(fn_name)?;
        hue_value_spaces!(space, fn_name, (self.c1, self.c2, self.c3))
    }

    /// `Premultiply::premultiply`：分量乘以 alpha，alpha 不变。
    ///
    /// `Premultiply::premultiply`: scale components by alpha; alpha stays as it is.
    pub fn premultiply(&self, fn_name: &str) -> Result<Self, ExtensionError> {
        let alpha = self.require_alpha01(fn_name)?;
        let space = self.require_space(fn_name)?;
        let a = (self.c1, self.c2, self.c3);
        let t = premultiply_spaces!(space, fn_name, a, alpha, true)?;
        Ok(Self::new(space, t.0, t.1, t.2, alpha))
    }

    /// `Premultiply::unpremultiply`：分量除以 alpha；alpha 为 0 时分量归零。
    ///
    /// `Premultiply::unpremultiply`: divide components by alpha; zero alpha zeros them.
    pub fn unpremultiply(&self, fn_name: &str) -> Result<Self, ExtensionError> {
        let alpha = self.require_alpha01(fn_name)?;
        let space = self.require_space(fn_name)?;
        let a = (self.c1, self.c2, self.c3);
        let t = premultiply_spaces!(space, fn_name, a, alpha, false)?;
        Ok(Self::new(space, t.0, t.1, t.2, alpha))
    }
}
