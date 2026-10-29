// ============================================================================
// 字符串互操作：palette 的十六进制读写 + csscolorparser 的 CSS Color Level 4。
//
// 两条来源不重复：
//   - `plt_from_hex` / `plt_to_hex` 是 palette 自己的 `Rgb::from_hex` / `LowerHex`（3/4/6/8 位，
//     alpha 用 4/8 位形式），单位与 palette 一样是 0..1；
//   - `plt_css_parse` / `plt_css_serialize` 走 csscolorparser，覆盖 CSS 的全部写法
//     （命名色、rgb()/hsl()/hwb()/lab()/lch()/oklab()/oklch()/color(...)），解析结果一律是 sRGB。
//
// 解析失败（不是合法颜色字符串）按三态契约返回 NULL；`plt_css_is_valid` 用来显式判断。
//
// String interop: palette's hex parsing/formatting plus csscolorparser's CSS Color Level 4 grammar
// (always producing sRGB). Unparsable text is NULL; `plt_css_is_valid` makes that checkable.
// ============================================================================

use csscolorparser::Color;
use duckfn::{DuckOptionResult, duck_scalar_function};

use palette::{Srgb, Srgba};
use quack_rs::error::ExtensionError;

use crate::extension::functions::err;
use crate::extension::types::color::PltColor;

/// 支持的序列化格式。
///
/// The supported serialization formats.
const CSS_FORMATS: &[&str] = &[
    "hex", "rgb", "hsl", "hwb", "lab", "lch", "oklab", "oklch",
];

/// 把 SQL 颜色转成 csscolorparser 的 `Color`（先转到 sRGB）。
///
/// Convert a SQL color into csscolorparser's `Color` (via sRGB).
fn to_css_color(color: &PltColor, fn_name: &str) -> Result<Color, ExtensionError> {
    let srgb = color.convert(fn_name, "srgb")?;
    Ok(Color::new(
        srgb.c1 as f32,
        srgb.c2 as f32,
        srgb.c3 as f32,
        srgb.alpha as f32,
    ))
}

/// 解析 CSS 颜色字符串，得到 sRGB 颜色。
#[duck_scalar_function(
    description = "Parse a CSS Color Level 4 string (named colors, #rgb/#rrggbb/#rrggbbaa, rgb(), hsl(), hwb(), lab(), lch(), oklab(), oklch(), color(...)) into an sRGB color; NULL when unparsable",
    example = "SELECT plt_css_parse('hsl(120 100% 50% / 0.5)')"
)]
fn plt_css_parse(text: String) -> DuckOptionResult<PltColor> {
    Ok(csscolorparser::parse(&text).ok().map(|c| {
        PltColor::new(
            "srgb",
            f64::from(c.r),
            f64::from(c.g),
            f64::from(c.b),
            f64::from(c.a),
        )
    }))
}

/// 字符串是否是合法的 CSS 颜色。
#[duck_scalar_function(
    description = "Whether a string is a valid CSS Color Level 4 color",
    example = "SELECT plt_css_is_valid('#ff0000')"
)]
fn plt_css_is_valid(text: String) -> bool {
    csscolorparser::parse(&text).is_ok()
}

/// 颜色最接近的 CSS 关键字名（精确匹配时才有）。
#[duck_scalar_function(
    description = "The CSS keyword name of a color when its sRGB bytes match a named color exactly, or NULL",
    example = "SELECT plt_css_name(plt_srgb(1, 0, 0))"
)]
fn plt_css_name(color: PltColor) -> DuckOptionResult<String> {
    let css = to_css_color(&color, "plt_css_name")?;
    Ok(css.name().map(|n| n.to_string()))
}

/// 把颜色序列化成 CSS 字符串。
#[duck_scalar_function(
    description = "Serialize a color into CSS syntax (format: hex, rgb, hsl, hwb, lab, lch, oklab, oklch)",
    example = "SELECT plt_css_serialize(plt_srgb(1, 0, 0), 'oklch')"
)]
fn plt_css_serialize(color: PltColor, format: String) -> DuckOptionResult<String> {
    let normalized = format.trim().to_ascii_lowercase().replace(['-', ' '], "_");
    if !CSS_FORMATS.contains(&normalized.as_str()) {
        return Err(err(format!(
            "plt_css_serialize: unknown format '{format}'; supported: {}",
            CSS_FORMATS.join(", ")
        )));
    }
    let css = to_css_color(&color, "plt_css_serialize")?;
    let text = match normalized.as_str() {
        "hex" => css.to_css_hex().to_string(),
        "rgb" => css.to_css_rgb().to_string(),
        "hsl" => css.to_css_hsl().to_string(),
        "hwb" => css.to_css_hwb().to_string(),
        "lab" => css.to_css_lab().to_string(),
        "lch" => css.to_css_lch().to_string(),
        "oklab" => css.to_css_oklab().to_string(),
        "oklch" => css.to_css_oklch().to_string(),
        _ => unreachable!("format was checked against CSS_FORMATS"),
    };
    Ok(Some(text))
}

/// 所有序列化格式名。
#[duck_scalar_function(
    description = "All supported CSS serialization format names, as accepted by plt_css_serialize",
    example = "SELECT plt_css_formats()"
)]
fn plt_css_formats() -> Vec<String> {
    CSS_FORMATS.iter().map(|f| f.to_string()).collect()
}

/// 解析十六进制颜色（palette 的 `from_hex`）。
#[duck_scalar_function(
    description = "Parse a hexadecimal color (#rgb, #rgba, #rrggbb, #rrggbbaa, with or without '#') using palette's hex parser; NULL when invalid",
    example = "SELECT plt_from_hex('#ff800080')"
)]
fn plt_from_hex(text: String) -> DuckOptionResult<PltColor> {
    use std::str::FromStr;
    if let Ok(rgba) = Srgba::<u8>::from_str(text.trim()) {
        return Ok(Some(PltColor::new(
            "srgb",
            f64::from(rgba.red) / 255.0,
            f64::from(rgba.green) / 255.0,
            f64::from(rgba.blue) / 255.0,
            f64::from(rgba.alpha) / 255.0,
        )));
    }
    if let Ok(rgb) = Srgb::<u8>::from_str(text.trim()) {
        return Ok(Some(PltColor::new(
            "srgb",
            f64::from(rgb.red) / 255.0,
            f64::from(rgb.green) / 255.0,
            f64::from(rgb.blue) / 255.0,
            1.0,
        )));
    }
    Ok(None)
}

/// 输出十六进制颜色（palette 的 `LowerHex`）。
#[duck_scalar_function(
    description = "Format a color as hexadecimal (#rrggbb, or #rrggbbaa when alpha is not 1) by converting to sRGB and quantizing to 8 bits",
    example = "SELECT plt_to_hex(plt_srgb(1, 0.5, 0))"
)]
fn plt_to_hex(color: PltColor) -> DuckOptionResult<String> {
    let srgb: Srgb<f64> = {
        let c = color.convert("plt_to_hex", "srgb")?;
        Srgb::new(c.c1, c.c2, c.c3)
    };
    let bytes: Srgb<u8> = srgb.into_format();
    if (color.alpha - 1.0).abs() < f64::EPSILON {
        Ok(Some(format!("#{bytes:02x}")))
    } else {
        let alpha: u8 = (color.alpha.clamp(0.0, 1.0) * 255.0).round() as u8;
        Ok(Some(format!("#{bytes:02x}{alpha:02x}")))
    }
}
