// ============================================================================
// CAM16 的自定义观察条件（palette 的 `cam16::Parameters` / `Surround` / `Discounting`）。
//
// `plt_color` 里的 `cam16_*` 空间固定用 palette 的默认条件（D65、La=40、Yb=0.2、average、
// auto），那是「随手转一下」的语义。需要显式控制观察条件时，走这里的 `plt_cam16_params`：
// 它把 `Parameters` 的五个字段（白点、环境亮度、背景亮度、周边、折扣）装进一个 STRUCT，
// 再由 `plt_cam16_from_xyz` / `plt_cam16_partial_from_xyz` 及其逆函数使用。
//
// 白点既可是名字（`plt_white_points()` 里的任意一个），也可是三个 XYZ 数值 —— 后者对应
// palette 的动态白点 `Parameters::default_dynamic_wp`。周边用百分比（0..20）表达，
// `dark` / `dim` / `average` 就是 0 / 10 / 20；折扣用 0..1 的数值，`auto` 时由 palette
// 按亮度与周边推算。越界的周边与折扣由 palette 自己夹紧（与上游文档一致）。
//
// 这些函数只接受 `xyz` 空间的颜色：分量被当作「该观察条件下白点的 XYZ 三刺激值」，返回的
// 颜色也标 `xyz` —— 白点不在值里，与 `plt_chromatic_adapt` 的约定相同。
//
// CAM16 with explicit viewing conditions. `plt_color`'s `cam16_*` spaces always use palette's
// defaults; `PltCam16Params` carries the five `Parameters` fields so the `plt_cam16_*` functions
// can use custom conditions. White points are given by name or as XYZ numbers (palette's dynamic
// white point), surround as a percentage (dark/dim/average = 0/10/20), discounting as 0..1 or
// auto. Inputs must be in the `xyz` space, matching `plt_chromatic_adapt`'s white-point convention.
// ============================================================================

use duckfn::DuckStruct;
use palette::cam16::{
    BakedParameters, Cam16, Cam16Jch, Cam16Jmh, Cam16Jsh, Cam16Qch, Cam16Qmh, Cam16Qsh,
    Cam16UcsJab, Cam16UcsJmh, Discounting, Parameters, Surround,
};
use palette::convert::FromColorUnclamped;
use palette::white_point::{Any, D65, WhitePoint};
use palette::Xyz;
use quack_rs::error::ExtensionError;

use super::adaptation;
use super::color::PltColor;
use crate::extension::functions::err;

/// 支持自定义观察条件的 CAM16 空间（与 `plt_color` 里的 `cam16_*` 同名）。
///
/// The CAM16 spaces that can be produced with custom viewing conditions.
pub const CAM16_SPACES: &[&str] = &[
    "cam16_jch",
    "cam16_jmh",
    "cam16_jsh",
    "cam16_qch",
    "cam16_qmh",
    "cam16_qsh",
    "cam16_ucs_jab",
    "cam16_ucs_jmh",
];

/// 周边条件的三个预设名。
///
/// The three named surround presets.
pub const SURROUNDS: &[&str] = &["dark", "dim", "average"];

/// palette `cam16::Parameters` 的五个字段，装成 SQL 结构体。
///
/// palette's `Parameters` fields as a SQL struct. Build it with `plt_cam16_params*` rather than
/// relying on the field order.
#[derive(Debug, Clone, DuckStruct)]
#[duck(sql_name = "plt_cam16_params", create_type = true)]
pub struct PltCam16Params {
    pub white_x: f64,
    pub white_y: f64,
    pub white_z: f64,
    pub adapting_luminance: f64,
    pub background_luminance: f64,
    pub surround: f64,
    pub discounting: f64,
    pub discounting_auto: bool,
}

impl Default for PltCam16Params {
    fn default() -> Self {
        Self::defaults()
    }
}

impl PltCam16Params {
    /// 转成 palette 的预烘焙参数，供多个颜色复用。
    ///
    /// Bake into palette's precomputed parameters, reusable across colors.
    pub fn baked(&self) -> BakedParameters<Xyz<Any, f64>, f64> {
        let mut parameters = Parameters::default_dynamic_wp(
            Xyz::<Any, f64>::new(self.white_x, self.white_y, self.white_z),
            self.adapting_luminance,
        );
        parameters.background_luminance = self.background_luminance;
        parameters.surround = Surround::Percent(self.surround);
        parameters.discounting = if self.discounting_auto {
            Discounting::Auto
        } else {
            Discounting::Custom(self.discounting)
        };
        parameters.bake()
    }

    /// palette 的默认观察条件：D65、La=40、Yb=0.2、average、auto。
    ///
    /// Palette's default viewing conditions.
    pub fn defaults() -> Self {
        let white = D65::get_xyz();
        Self {
            white_x: white.x,
            white_y: white.y,
            white_z: white.z,
            adapting_luminance: 40.0,
            background_luminance: 0.2,
            surround: 20.0,
            discounting: 1.0,
            discounting_auto: true,
        }
    }
}

/// 周边名 → 百分比；也接受直接写数值（如 `'12.5'`）。
///
/// Surround name into its percentage; a number written as text (`'12.5'`) is accepted too.
pub fn surround_percent(fn_name: &str, surround: &str) -> Result<f64, ExtensionError> {
    let normalized = surround.trim().to_ascii_lowercase().replace(['-', ' '], "_");
    match normalized.as_str() {
        "dark" => Ok(0.0),
        "dim" => Ok(10.0),
        "average" => Ok(20.0),
        _ => normalized.parse::<f64>().map_err(|_| {
            err(format!(
                "{fn_name}: unknown surround '{surround}'; use dark, dim, average, or a percentage like 12.5"
            ))
        }),
    }
}

/// 组装参数结构体（白点数值已给定）。
///
/// Assemble the parameters from an XYZ white point and the remaining fields.
pub fn params(
    fn_name: &str,
    white: (f64, f64, f64),
    adapting_luminance: f64,
    background_luminance: f64,
    surround: &str,
    discounting: Option<f64>,
) -> Result<PltCam16Params, ExtensionError> {
    let surround = surround_percent(fn_name, surround)?;
    Ok(PltCam16Params {
        white_x: white.0,
        white_y: white.1,
        white_z: white.2,
        adapting_luminance,
        background_luminance,
        surround,
        discounting: discounting.unwrap_or(1.0),
        discounting_auto: discounting.is_none(),
    })
}

/// 按名字取白点后组装。
///
/// Assemble the parameters from a named white point.
pub fn params_named(
    fn_name: &str,
    white_point: &str,
    adapting_luminance: f64,
    background_luminance: f64,
    surround: &str,
    discounting: Option<f64>,
) -> Result<PltCam16Params, ExtensionError> {
    let white = adaptation::white_point_xyz(fn_name, white_point)?;
    params(
        fn_name,
        white,
        adapting_luminance,
        background_luminance,
        surround,
        discounting,
    )
}

/// 取出 `xyz` 空间的三刺激值；其它空间给出带转换建议的错误。
///
/// Pull the tristimulus values out of an `xyz` color; other spaces get a hint to convert first.
pub fn require_xyz(color: &PltColor, fn_name: &str) -> Result<Xyz<Any, f64>, ExtensionError> {
    let space = color.require_space(fn_name)?;
    if space != "xyz" {
        return Err(err(format!(
            "{fn_name}: expected a color in the 'xyz' space (its components are tristimulus values under the parameters' white point), got '{space}'; convert it first, e.g. plt_convert(color, 'xyz')"
        )));
    }
    Ok(Xyz::<Any, f64>::new(color.c1, color.c2, color.c3))
}

/// 任意 `xyz` 颜色 → CAM16 全属性。
///
/// Any `xyz` color into the full set of CAM16 attributes.
pub fn full_from_xyz(
    color: &PltColor,
    fn_name: &str,
    parameters: &PltCam16Params,
) -> Result<Cam16<f64>, ExtensionError> {
    let xyz = require_xyz(color, fn_name)?;
    Ok(Cam16::from_xyz(xyz, parameters.baked()))
}

/// CAM16 全属性 → `xyz` 颜色（alpha 由调用方带入）。
///
/// The full CAM16 attributes back into an `xyz` color.
pub fn full_to_xyz(cam16: Cam16<f64>, parameters: &PltCam16Params) -> PltColor {
    let xyz = cam16.into_xyz(parameters.baked());
    PltColor::new("xyz", xyz.x, xyz.y, xyz.z, 1.0)
}

/// 任意 `xyz` 颜色 → 某个 CAM16 部分空间（`cam16_jch` … `cam16_ucs_jmh`）。
///
/// Any `xyz` color into one of the partial CAM16 spaces.
pub fn partial_from_xyz(
    color: &PltColor,
    fn_name: &str,
    space: &str,
    parameters: &PltCam16Params,
) -> Result<PltColor, ExtensionError> {
    let xyz = require_xyz(color, fn_name)?;
    let parameters = parameters.baked();
    let space = super::color::space_arg(fn_name, space)?;
    let (c1, c2, c3) = match space {
        "cam16_jch" => {
            let c = Cam16Jch::from_xyz(xyz, parameters);
            (c.lightness, c.chroma, c.hue.into_positive_degrees())
        }
        "cam16_jmh" => {
            let c = Cam16Jmh::from_xyz(xyz, parameters);
            (c.lightness, c.colorfulness, c.hue.into_positive_degrees())
        }
        "cam16_jsh" => {
            let c = Cam16Jsh::from_xyz(xyz, parameters);
            (c.lightness, c.saturation, c.hue.into_positive_degrees())
        }
        "cam16_qch" => {
            let c = Cam16Qch::from_xyz(xyz, parameters);
            (c.brightness, c.chroma, c.hue.into_positive_degrees())
        }
        "cam16_qmh" => {
            let c = Cam16Qmh::from_xyz(xyz, parameters);
            (c.brightness, c.colorfulness, c.hue.into_positive_degrees())
        }
        "cam16_qsh" => {
            let c = Cam16Qsh::from_xyz(xyz, parameters);
            (c.brightness, c.saturation, c.hue.into_positive_degrees())
        }
        "cam16_ucs_jab" => {
            let jmh = Cam16Jmh::from_xyz(xyz, parameters);
            Cam16UcsJab::from_color_unclamped(jmh).into_components()
        }
        "cam16_ucs_jmh" => {
            let jmh = Cam16Jmh::from_xyz(xyz, parameters);
            let c = Cam16UcsJmh::from_color_unclamped(jmh);
            (c.lightness, c.colorfulness, c.hue.into_positive_degrees())
        }
        other => {
            return Err(err(format!(
                "{fn_name}: '{}' is not a CAM16 space; use one of {}",
                other,
                CAM16_SPACES.join(", ")
            )))
        }
    };
    Ok(color.with_components(space, c1, c2, c3))
}

/// 某个 CAM16 部分空间 → `xyz` 颜色。
///
/// One of the partial CAM16 spaces back into an `xyz` color.
pub fn partial_to_xyz(
    color: &PltColor,
    fn_name: &str,
    parameters: &PltCam16Params,
) -> Result<PltColor, ExtensionError> {
    let space = color.require_space(fn_name)?;
    let parameters = parameters.baked();
    let (c1, c2, c3) = (color.c1, color.c2, color.c3);
    let xyz = match space {
        "cam16_jch" => Cam16Jch::new(c1, c2, c3).into_xyz(parameters),
        "cam16_jmh" => Cam16Jmh::new(c1, c2, c3).into_xyz(parameters),
        "cam16_jsh" => Cam16Jsh::new(c1, c2, c3).into_xyz(parameters),
        "cam16_qch" => Cam16Qch::new(c1, c2, c3).into_xyz(parameters),
        "cam16_qmh" => Cam16Qmh::new(c1, c2, c3).into_xyz(parameters),
        "cam16_qsh" => Cam16Qsh::new(c1, c2, c3).into_xyz(parameters),
        "cam16_ucs_jab" => {
            let jmh = Cam16Jmh::from_color_unclamped(Cam16UcsJab::new(c1, c2, c3));
            jmh.into_xyz(parameters)
        }
        "cam16_ucs_jmh" => {
            let jmh = Cam16Jmh::from_color_unclamped(Cam16UcsJmh::new(c1, c2, c3));
            jmh.into_xyz(parameters)
        }
        other => {
            return Err(err(format!(
                "{fn_name}: expected a CAM16 space ({}), got '{other}'",
                CAM16_SPACES.join(", ")
            )))
        }
    };
    Ok(color.with_components("xyz", xyz.x, xyz.y, xyz.z))
}
