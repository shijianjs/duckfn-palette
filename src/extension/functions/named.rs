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
use palette::color_difference::Ciede2000;
use palette::convert::FromColorUnclamped;
use palette::white_point::D65;
use palette::{Lab, Srgb};
use std::sync::OnceLock;

use crate::extension::types::color::PltColor;

/// 148 个命名色预先转成 CIE L*a*b*，供最近名查找使用。
///
/// The 148 named colors pre-converted to CIE L*a*b* for the nearest-name lookup.
fn named_lab_table() -> &'static [(&'static str, Lab<D65, f64>)] {
    static TABLE: OnceLock<Vec<(&'static str, Lab<D65, f64>)>> = OnceLock::new();
    TABLE.get_or_init(|| {
        palette::named::entries()
            .map(|(name, color)| {
                let rgb: Srgb<f64> = color.into_format();
                (name, Lab::<D65, f64>::from_color_unclamped(rgb))
            })
            .collect()
    })
}

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

/// 最接近的命名色（CIEDE2000 距离）。
#[duck_scalar_function(
    description = "The closest SVG/CSS3 named color by CIEDE2000 distance, regardless of exact match; alpha is ignored",
    example = "SELECT plt_nearest_named(plt_srgb(0.95, 0.05, 0.05))"
)]
fn plt_nearest_named(color: PltColor) -> DuckOptionResult<String> {
    let lab = color.convert("plt_nearest_named", "lab")?;
    let target = Lab::<D65, f64>::new(lab.c1, lab.c2, lab.c3);
    let mut best: Option<(&'static str, f64)> = None;
    for &(name, named) in named_lab_table() {
        let distance = target.difference(named);
        let better = match best {
            None => true,
            Some((best_name, best_distance)) => {
                distance < best_distance || (distance == best_distance && name < best_name)
            }
        };
        if better {
            best = Some((name, distance));
        }
    }
    Ok(best.map(|(name, _)| name.to_string()))
}
