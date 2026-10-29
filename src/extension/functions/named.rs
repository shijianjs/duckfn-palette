// ============================================================================
// 命名色（palette 的 `named` 模块）。
//
// palette 的命名色是 SVG / CSS3 关键字色，一共 148 个，存成 `Srgb<u8>`。这里按名字查表、
// 列出全部名字；认不出的名字按三态契约返回 NULL（与 `named::from_str` 的 `Option` 一致）。
//
// Named colors: palette's 148 SVG/CSS3 keyword colors. Unknown names are NULL, matching
// `named::from_str`'s `Option`.
// ============================================================================

use duckfn::{DuckOptionResult, duck_scalar_function};

use crate::extension::types::color::PltColor;

/// 按名字取命名色（小写）。
#[duck_scalar_function(
    description = "An SVG/CSS3 named color such as 'rebeccapurple' as an sRGB color, or NULL for an unknown name",
    example = "SELECT plt_named('rebeccapurple')"
)]
fn plt_named(name: String) -> DuckOptionResult<PltColor> {
    let color = palette::named::from_str(&name.trim().to_ascii_lowercase());
    Ok(color.map(|c| {
        PltColor::new(
            "srgb",
            f64::from(c.red) / 255.0,
            f64::from(c.green) / 255.0,
            f64::from(c.blue) / 255.0,
            1.0,
        )
    }))
}

/// 全部命名色的名字（按字母序）。
#[duck_scalar_function(
    description = "All SVG/CSS3 named color names, sorted alphabetically",
    example = "SELECT plt_named_names()"
)]
fn plt_named_names() -> Vec<String> {
    let mut names: Vec<String> = palette::named::names().map(|n| n.to_string()).collect();
    names.sort();
    names
}
