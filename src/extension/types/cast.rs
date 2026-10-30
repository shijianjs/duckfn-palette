// ============================================================================
// 打包/解包（palette 的 `cast::Packed`，即 `PackedArgb` / `PackedRgba` / `PackedAbgr` /
// `PackedBgra`）。
//
// 颜色先转到 sRGB、按 8 位量化（`into_format::<u8>`：四舍五入并夹到 0..255），再按通道顺序
// 打进一个 u32；解包是逆过程。`argb` 是 palette 的默认顺序（6 位十六进制数 `0xRRGGBB`
// 读起来最自然），其余三种供对接其它格式。
//
// Packing in palette's `cast::Packed` sense: quantize sRGB components and alpha to 8 bits, then
// put them into a u32 in the requested channel order; unpacking reverses it. `argb` is palette's
// default (6-digit `0xRRGGBB` reads naturally); the other orders are for external formats.
// ============================================================================

use palette::Srgba;
use palette::cast::Packed;
use palette::rgb::channels::{Abgr, Argb, Bgra, Rgba};
use quack_rs::error::ExtensionError;

use super::color::{PltColor, bad_arg};

/// 支持的通道顺序（规范形式）。
///
/// Supported channel orders, normalized.
pub const PACK_ORDERS: &[&str] = &["argb", "rgba", "abgr", "bgra"];

/// 通道顺序解析。
///
/// Resolve a channel order name.
pub fn pack_order(fn_name: &str, order: &str) -> Result<&'static str, ExtensionError> {
    let normalized = order.trim().to_ascii_lowercase().replace(['-', ' '], "_");
    PACK_ORDERS
        .iter()
        .copied()
        .find(|o| *o == normalized)
        .ok_or_else(|| {
            bad_arg(
                fn_name,
                format!(
                    "unknown channel order '{order}'; supported: {}",
                    PACK_ORDERS.join(", ")
                ),
            )
        })
}

/// 把颜色打包成 u32。
///
/// Pack a color into a u32.
pub fn pack(fn_name: &str, color: &PltColor, order: &str) -> Result<i64, ExtensionError> {
    let order = pack_order(fn_name, order)?;
    let srgb = color.convert_clamped(fn_name, "srgb")?;
    let rgba = Srgba::<f64>::new(srgb.c1, srgb.c2, srgb.c3, color.alpha);
    let rgba = rgba.into_format::<u8, u8>();
    let packed = match order {
        "argb" => Packed::<Argb, u32>::pack(rgba).color,
        "rgba" => Packed::<Rgba, u32>::pack(rgba).color,
        "abgr" => Packed::<Abgr, u32>::pack(rgba).color,
        "bgra" => Packed::<Bgra, u32>::pack(rgba).color,
        _ => unreachable!("pack_order returned an unknown order"),
    };
    Ok(i64::from(packed))
}

/// 把 u32 解包成 sRGB 颜色。
///
/// Unpack a u32 into an sRGB color.
pub fn unpack(fn_name: &str, value: i64, order: &str) -> Result<PltColor, ExtensionError> {
    let order = pack_order(fn_name, order)?;
    let packed = u32::try_from(value).map_err(|_| {
        bad_arg(
            fn_name,
            format!("packed value must be within 0..4294967295, got {value}"),
        )
    })?;
    let rgba = match order {
        "argb" => Packed::<Argb, u32> {
            color: packed,
            channel_order: std::marker::PhantomData,
        }
        .unpack::<Srgba<u8>>(),
        "rgba" => Packed::<Rgba, u32> {
            color: packed,
            channel_order: std::marker::PhantomData,
        }
        .unpack::<Srgba<u8>>(),
        "abgr" => Packed::<Abgr, u32> {
            color: packed,
            channel_order: std::marker::PhantomData,
        }
        .unpack::<Srgba<u8>>(),
        "bgra" => Packed::<Bgra, u32> {
            color: packed,
            channel_order: std::marker::PhantomData,
        }
        .unpack::<Srgba<u8>>(),
        _ => unreachable!("pack_order returned an unknown order"),
    };
    let rgba = rgba.into_format::<f64, f64>();
    Ok(PltColor::new(
        "srgb", rgba.red, rgba.green, rgba.blue, rgba.alpha,
    ))
}
