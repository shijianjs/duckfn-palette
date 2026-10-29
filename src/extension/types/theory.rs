// ============================================================================
// 传统色彩理论（palette 的 `color_theory` 模块）：互补、分裂互补、邻近、三角、四方。
//
// palette 把这些概念定义成对 `ShiftHue + Clone` 的 blanket 实现 —— 也就是说只有带色相的空间
// （hsl / hsv / hwb / hsluv / lch / lchuv / oklch / okhsl / okhsv / okhwb / 6 个 CAM16 部分
// 空间 / cam16_ucs_jmh）能用。这里复用 palette 的 trait 本身，按空间分派，最后把色相按
// `SqlComponents` 的规则归一到 0..360。
//
// Traditional color theory: complementary, split-complementary, analogous, triadic and tetradic
// schemes. palette defines these as blanket impls over `ShiftHue + Clone`, i.e. the hue spaces.
// The traits themselves are called per space; hues are normalized on the way out.
// ============================================================================

use palette::ShiftHue;
use palette::angle::HalfRotation;
use palette::cam16::{Cam16Jch, Cam16Jmh, Cam16Jsh, Cam16Qch, Cam16Qmh, Cam16Qsh, Cam16UcsJmh};
use palette::color_theory::{Analogous, Complementary, SplitComplementary, Tetradic, Triadic};
use palette::encoding;
use palette::num::Real;
use palette::white_point::D65;
use palette::{Hsl, Hsluv, Hsv, Hwb, Lch, Lchuv, Okhsl, Okhsv, Okhwb, Oklch};
use quack_rs::error::ExtensionError;

use super::color::{PltColor, bad_arg};
use super::ops::SqlComponents;

/// 一种色彩理论配色方案。
///
/// One color theory scheme.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scheme {
    Complementary,
    SplitComplementary,
    Analogous,
    AnalogousSecondary,
    Triadic,
    Tetradic,
}

/// 所有方案名（规范化后）。
///
/// Every scheme name, normalized.
pub const SCHEMES: &[&str] = &[
    "complementary",
    "split_complementary",
    "analogous",
    "analogous_secondary",
    "triadic",
    "tetradic",
];

/// 解析方案名。
///
/// Resolve a scheme name.
pub fn scheme_arg(fn_name: &str, scheme: &str) -> Result<Scheme, ExtensionError> {
    let normalized = scheme.trim().to_ascii_lowercase().replace(['-', ' '], "_");
    let resolved = match normalized.as_str() {
        "complementary" | "complement" => Scheme::Complementary,
        "split_complementary" | "split_complement" => Scheme::SplitComplementary,
        "analogous" => Scheme::Analogous,
        "analogous_secondary" => Scheme::AnalogousSecondary,
        "triadic" | "triad" => Scheme::Triadic,
        "tetradic" | "square" => Scheme::Tetradic,
        other => {
            return Err(bad_arg(
                fn_name,
                format!(
                    "unknown color scheme '{other}'; supported: {}",
                    SCHEMES.join(", ")
                ),
            ));
        }
    };
    Ok(resolved)
}

/// 对任意带色相的颜色求某个方案（palette 的 trait 都是 blanket 实现）。
///
/// Apply a scheme to any hue-bearing color; palette's traits are blanket impls.
fn harmonic<C>(color: C, scheme: Scheme) -> Vec<C>
where
    C: ShiftHue + Clone,
    C::Scalar: Real + HalfRotation,
{
    match scheme {
        Scheme::Complementary => vec![color.complementary()],
        Scheme::SplitComplementary => {
            let (first, second) = color.split_complementary();
            vec![first, second]
        }
        Scheme::Analogous => {
            let (first, second) = color.analogous();
            vec![first, second]
        }
        Scheme::AnalogousSecondary => {
            let (first, second) = color.analogous_secondary();
            vec![first, second]
        }
        Scheme::Triadic => {
            let (first, second) = color.triadic();
            vec![first, second]
        }
        Scheme::Tetradic => {
            let (first, second, third) = color.tetradic();
            vec![first, second, third]
        }
    }
}

/// 同 [`harmonic`]，但结果摊成 SQL 侧的三元组。
///
/// Same as [`harmonic`], flattened into SQL-side triples.
fn harmonic_triples<C>(color: C, scheme: Scheme) -> Vec<(f64, f64, f64)>
where
    C: ShiftHue + Clone + SqlComponents,
    C::Scalar: Real + HalfRotation,
{
    harmonic(color, scheme)
        .into_iter()
        .map(|c| c.to_triple())
        .collect()
}

/// 空间不支持色彩理论方案时的统一错误。
///
/// Uniform "space has no hue, so no scheme" error.
fn unsupported(fn_name: &str, space: &str) -> ExtensionError {
    bad_arg(
        fn_name,
        format!(
            "color space '{space}' has no hue, so it has no color theory schemes; \
             convert it first, e.g. plt_convert(color, 'oklch')"
        ),
    )
}

/// 一个方案在全部带色相空间上的分派。
///
/// One scheme dispatched over every hue space.
macro_rules! theory_dispatch {
    ($space:expr, $fn_name:expr, $a:expr, $scheme:expr) => {{
        let a = $a;
        match $space {
            "hsl" => Ok(harmonic_triples(Hsl::<encoding::Srgb, f64>::from_triple(a), $scheme)),
            "hsv" => Ok(harmonic_triples(Hsv::<encoding::Srgb, f64>::from_triple(a), $scheme)),
            "hwb" => Ok(harmonic_triples(Hwb::<encoding::Srgb, f64>::from_triple(a), $scheme)),
            "hsluv" => Ok(harmonic_triples(Hsluv::<D65, f64>::from_triple(a), $scheme)),
            "lch" => Ok(harmonic_triples(Lch::<D65, f64>::from_triple(a), $scheme)),
            "lchuv" => Ok(harmonic_triples(Lchuv::<D65, f64>::from_triple(a), $scheme)),
            "oklch" => Ok(harmonic_triples(Oklch::<f64>::from_triple(a), $scheme)),
            "okhsl" => Ok(harmonic_triples(Okhsl::<f64>::from_triple(a), $scheme)),
            "okhsv" => Ok(harmonic_triples(Okhsv::<f64>::from_triple(a), $scheme)),
            "okhwb" => Ok(harmonic_triples(Okhwb::<f64>::from_triple(a), $scheme)),
            "cam16_jch" => Ok(harmonic_triples(Cam16Jch::<f64>::from_triple(a), $scheme)),
            "cam16_jmh" => Ok(harmonic_triples(Cam16Jmh::<f64>::from_triple(a), $scheme)),
            "cam16_jsh" => Ok(harmonic_triples(Cam16Jsh::<f64>::from_triple(a), $scheme)),
            "cam16_qch" => Ok(harmonic_triples(Cam16Qch::<f64>::from_triple(a), $scheme)),
            "cam16_qmh" => Ok(harmonic_triples(Cam16Qmh::<f64>::from_triple(a), $scheme)),
            "cam16_qsh" => Ok(harmonic_triples(Cam16Qsh::<f64>::from_triple(a), $scheme)),
            "cam16_ucs_jmh" => Ok(harmonic_triples(Cam16UcsJmh::<f64>::from_triple(a), $scheme)),
            other => Err(unsupported($fn_name, other)),
        }
    }};
}

impl PltColor {
    /// 某个色彩理论方案给出的其余颜色（不含输入色本身）。
    ///
    /// The other colors of a color theory scheme, not including the input color.
    pub fn harmony(&self, scheme: &str, fn_name: &str) -> Result<Vec<Self>, ExtensionError> {
        let scheme = scheme_arg(fn_name, scheme)?;
        let space = self.require_space(fn_name)?;
        let a = (self.c1, self.c2, self.c3);
        let hues: Vec<(f64, f64, f64)> = theory_dispatch!(space, fn_name, a, scheme)?;
        Ok(hues
            .into_iter()
            .map(|(c1, c2, c3)| self.with_components(space, c1, c2, c3))
            .collect())
    }
}
