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
const CSS_FORMATS: &[&str] = &["hex", "rgb", "hsl", "hwb", "lab", "lch", "oklab", "oklch"];

/// 把颜色转成 sRGB 的 8 位分量与 alpha 字节（写 hex 用）。
///
/// Convert a color into 8-bit sRGB components and an alpha byte for hex output.
fn hex_parts(color: &PltColor, fn_name: &str) -> Result<(Srgb<u8>, u8), ExtensionError> {
    let srgb = color.convert(fn_name, "srgb")?;
    let bytes: Srgb<u8> = Srgb::<f64>::new(srgb.c1, srgb.c2, srgb.c3).into_format();
    let alpha: u8 = (color.alpha.clamp(0.0, 1.0) * 255.0).round() as u8;
    Ok((bytes, alpha))
}

/// alpha 按两位小数显示（`0.5`、`0`、`1` 这样的短形式）。
///
/// Round alpha to two decimals for CSS text, without trailing zeros.
fn display_alpha(alpha: f64) -> f64 {
    (alpha.clamp(0.0, 1.0) * 100.0).round() / 100.0
}

/// 是否按「不透明」处理（与 `plt_to_hex` 的判定一致）。
///
/// Whether alpha counts as opaque, matching `plt_to_hex`.
fn is_opaque(alpha: f64) -> bool {
    (alpha - 1.0).abs() < f64::EPSILON
}

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
    let (bytes, alpha) = hex_parts(&color, "plt_to_hex")?;
    if is_opaque(color.alpha) {
        Ok(Some(format!("#{bytes:02x}")))
    } else {
        Ok(Some(format!("#{bytes:02x}{alpha:02x}")))
    }
}

/// 输出固定 6 位十六进制（忽略 alpha）。
#[duck_scalar_function(
    description = "Format a color as #rrggbb, always six digits; alpha is ignored",
    example = "SELECT plt_to_hex6(plt_color('srgb', 1, 0.5, 0, 0.5))"
)]
fn plt_to_hex6(color: PltColor) -> DuckOptionResult<String> {
    let (bytes, _) = hex_parts(&color, "plt_to_hex6")?;
    Ok(Some(format!("#{bytes:02x}")))
}

/// 输出固定 8 位十六进制（总是带 alpha）。
#[duck_scalar_function(
    description = "Format a color as #rrggbbaa, always eight digits (alpha = ff when opaque)",
    example = "SELECT plt_to_hex8(plt_srgb(1, 0.5, 0))"
)]
fn plt_to_hex8(color: PltColor) -> DuckOptionResult<String> {
    let (bytes, alpha) = hex_parts(&color, "plt_to_hex8")?;
    Ok(Some(format!("#{bytes:02x}{alpha:02x}")))
}

/// 输出百分比形式的 rgb()/rgba() 字符串。
#[duck_scalar_function(
    description = "Format a color as CSS rgb() with percentages, or rgba() when alpha is not 1; channels and alpha are rounded like tinycolor",
    example = "SELECT plt_to_percentage_rgb(plt_srgb(1, 0.5, 0))"
)]
fn plt_to_percentage_rgb(color: PltColor) -> DuckOptionResult<String> {
    let srgb = color.convert("plt_to_percentage_rgb", "srgb")?;
    let percent = |value: f64| (value.clamp(0.0, 1.0) * 100.0).round() as i64;
    let (r, g, b) = (percent(srgb.c1), percent(srgb.c2), percent(srgb.c3));
    if is_opaque(color.alpha) {
        Ok(Some(format!("rgb({r}%, {g}%, {b}%)")))
    } else {
        Ok(Some(format!(
            "rgba({r}%, {g}%, {b}%, {})",
            display_alpha(color.alpha)
        )))
    }
}

/// 输出 hsv()/hsva() 字符串（CSS 没有 HSV，但 tinycolor 用这套写法）。
#[duck_scalar_function(
    description = "Format a color as hsv(h, s%, v%), or hsva() when alpha is not 1; hue and percentages are rounded like tinycolor",
    example = "SELECT plt_to_hsv_string(plt_srgb(1, 0, 0))"
)]
fn plt_to_hsv_string(color: PltColor) -> DuckOptionResult<String> {
    let hsv = color.convert("plt_to_hsv_string", "hsv")?;
    let hue = hsv.c1.round() as i64;
    let saturation = (hsv.c2.clamp(0.0, 1.0) * 100.0).round() as i64;
    let value = (hsv.c3.clamp(0.0, 1.0) * 100.0).round() as i64;
    if is_opaque(color.alpha) {
        Ok(Some(format!("hsv({hue}, {saturation}%, {value}%)")))
    } else {
        Ok(Some(format!(
            "hsva({hue}, {saturation}%, {value}%, {})",
            display_alpha(color.alpha)
        )))
    }
}
