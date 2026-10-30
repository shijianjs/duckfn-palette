// ============================================================================
// colorous 的预设配色方案（d3-scale-chromatic 的移植）。
//
// colorous 有两类东西：**连续渐变**（`Gradient`，如 viridis、spectral）和**分类色板**
// （固定长度的 `[Color; N]`，如 category10、tableau10）。两者在这里都按小写名字暴露，
// 名字就是 colorous 常量名的小写形式（`BLUE_GREEN` → `blue_green`）。
//
// 渐变可用 `eval_continuous(t)`（t 夹到 0..1）或 `eval_rational(i, n)`（等价于
// `t = i/(n-1)`，取第 i 个分位）采样；分类色板只能整组取。
//
// colorous' presets: continuous gradients (viridis, spectral, ...) and fixed categorical arrays
// (category10, tableau10, ...). Names are the lowercase constant names. Gradients sample at a
// continuous position or at a rational index; categorical palettes are returned whole.
// ============================================================================

use colorous::{Color, Gradient};
use quack_rs::error::ExtensionError;

use super::color::PltColor;
use crate::extension::functions::err;

/// 所有连续渐变的名字（与 colorous 常量一一对应）。
///
/// Every continuous gradient name.
pub const GRADIENTS: &[&str] = &[
    // sequential (multi-hue)
    "turbo",
    "viridis",
    "inferno",
    "magma",
    "plasma",
    "cividis",
    "warm",
    "cool",
    "cubehelix",
    "blue_green",
    "blue_purple",
    "green_blue",
    "orange_red",
    "purple_blue_green",
    "purple_blue",
    "purple_red",
    "red_purple",
    "yellow_green_blue",
    "yellow_green",
    "yellow_orange_brown",
    "yellow_orange_red",
    // sequential (single-hue)
    "blues",
    "greens",
    "greys",
    "oranges",
    "purples",
    "reds",
    // diverging
    "brown_green",
    "pink_green",
    "purple_green",
    "purple_orange",
    "red_blue",
    "red_grey",
    "red_yellow_blue",
    "red_yellow_green",
    "spectral",
    // cyclical
    "rainbow",
    "sinebow",
];

/// 所有分类色板的名字。
///
/// Every categorical palette name.
pub const CATEGORICALS: &[&str] = &[
    "category10",
    "accent",
    "dark2",
    "paired",
    "pastel1",
    "pastel2",
    "set1",
    "set2",
    "set3",
    "tableau10",
];

/// 名字规范化：大小写不敏感，`-` / 空格视作 `_`。
///
/// Normalize a name: case-insensitive, `-` and spaces become `_`.
fn normalize(name: &str) -> String {
    name.trim().to_ascii_lowercase().replace(['-', ' '], "_")
}

/// 按名字取连续渐变。
///
/// Resolve a gradient by name.
pub fn gradient(fn_name: &str, name: &str) -> Result<Gradient, ExtensionError> {
    let gradient = match normalize(name).as_str() {
        "turbo" => colorous::TURBO,
        "viridis" => colorous::VIRIDIS,
        "inferno" => colorous::INFERNO,
        "magma" => colorous::MAGMA,
        "plasma" => colorous::PLASMA,
        "cividis" => colorous::CIVIDIS,
        "warm" => colorous::WARM,
        "cool" => colorous::COOL,
        "cubehelix" => colorous::CUBEHELIX,
        "blue_green" => colorous::BLUE_GREEN,
        "blue_purple" => colorous::BLUE_PURPLE,
        "green_blue" => colorous::GREEN_BLUE,
        "orange_red" => colorous::ORANGE_RED,
        "purple_blue_green" => colorous::PURPLE_BLUE_GREEN,
        "purple_blue" => colorous::PURPLE_BLUE,
        "purple_red" => colorous::PURPLE_RED,
        "red_purple" => colorous::RED_PURPLE,
        "yellow_green_blue" => colorous::YELLOW_GREEN_BLUE,
        "yellow_green" => colorous::YELLOW_GREEN,
        "yellow_orange_brown" => colorous::YELLOW_ORANGE_BROWN,
        "yellow_orange_red" => colorous::YELLOW_ORANGE_RED,
        "blues" => colorous::BLUES,
        "greens" => colorous::GREENS,
        "greys" | "grays" => colorous::GREYS,
        "oranges" => colorous::ORANGES,
        "purples" => colorous::PURPLES,
        "reds" => colorous::REDS,
        "brown_green" => colorous::BROWN_GREEN,
        "pink_green" => colorous::PINK_GREEN,
        "purple_green" => colorous::PURPLE_GREEN,
        "purple_orange" => colorous::PURPLE_ORANGE,
        "red_blue" => colorous::RED_BLUE,
        "red_grey" | "red_gray" => colorous::RED_GREY,
        "red_yellow_blue" => colorous::RED_YELLOW_BLUE,
        "red_yellow_green" => colorous::RED_YELLOW_GREEN,
        "spectral" => colorous::SPECTRAL,
        "rainbow" => colorous::RAINBOW,
        "sinebow" => colorous::SINEBOW,
        other => {
            return Err(err(format!(
                "{fn_name}: unknown gradient '{other}'; see plt_colorous_palettes() for the {} names",
                GRADIENTS.len()
            )));
        }
    };
    Ok(gradient)
}

/// 按名字取分类色板。
///
/// Resolve a categorical palette by name.
pub fn categorical(fn_name: &str, name: &str) -> Result<&'static [Color], ExtensionError> {
    let palette: &'static [Color] = match normalize(name).as_str() {
        "category10" => &colorous::CATEGORY10,
        "accent" => &colorous::ACCENT,
        "dark2" => &colorous::DARK2,
        "paired" => &colorous::PAIRED,
        "pastel1" => &colorous::PASTEL1,
        "pastel2" => &colorous::PASTEL2,
        "set1" => &colorous::SET1,
        "set2" => &colorous::SET2,
        "set3" => &colorous::SET3,
        "tableau10" => &colorous::TABLEAU10,
        other => {
            return Err(err(format!(
                "{fn_name}: unknown categorical palette '{other}'; see plt_colorous_categoricals()"
            )));
        }
    };
    Ok(palette)
}

/// colorous 的 8 位 sRGB 颜色 → `plt_color`（分量除以 255）。
///
/// A colorous 8-bit sRGB color as `plt_color`.
pub fn to_color(c: Color) -> PltColor {
    PltColor::new(
        "srgb",
        f64::from(c.r) / 255.0,
        f64::from(c.g) / 255.0,
        f64::from(c.b) / 255.0,
        1.0,
    )
}
