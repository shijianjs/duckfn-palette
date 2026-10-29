// ============================================================================
// 打包/解包（palette 的 `cast::Packed`）。
//
// 颜色先转 sRGB、量化到 8 位，再按通道顺序打进一个 u32（SQL 侧是 BIGINT，范围 0..4294967295）。
// `argb` 是 palette 的默认顺序，`plt_pack(color)` 默认用它；四个顺序见 `plt_pack_orders()`。
// `plt_unpack` 是逆过程，结果总是 `srgb` 颜色（alpha 一并还原）。
//
// Pack/unpack in palette's `cast::Packed` sense: quantize to 8-bit sRGB + alpha and pack into a
// u32 (BIGINT in SQL). `argb` is palette's default; `plt_unpack` always yields an `srgb` color.
// ============================================================================

use duckfn::{DuckOptionResult, duck_scalar_function};

use crate::extension::types::cast;
use crate::extension::types::color::PltColor;

/// 支持的通道顺序。
#[duck_scalar_function(
    description = "Channel orders accepted by plt_pack and plt_unpack (argb, rgba, abgr, bgra)",
    example = "SELECT plt_pack_orders()"
)]
fn plt_pack_orders() -> Vec<String> {
    cast::PACK_ORDERS.iter().map(|o| o.to_string()).collect()
}

/// 按 ARGB 顺序打包（palette 的默认顺序）。
#[duck_scalar_function(
    overloads_name = "plt_pack",
    description = "Pack a color into a 32-bit ARGB integer after converting to sRGB and quantizing to 8 bits per channel",
    example = "SELECT plt_pack(plt_srgb(1, 0.5, 0))"
)]
fn plt_pack_1(color: PltColor) -> DuckOptionResult<i64> {
    Ok(Some(cast::pack("plt_pack", &color, "argb")?))
}

/// 按指定通道顺序打包。
#[duck_scalar_function(
    overloads_name = "plt_pack",
    description = "Pack a color into a 32-bit integer in the given channel order (argb, rgba, abgr, bgra)",
    example = "SELECT plt_pack(plt_srgb(1, 0.5, 0), 'rgba')"
)]
fn plt_pack_2(color: PltColor, order: String) -> DuckOptionResult<i64> {
    Ok(Some(cast::pack("plt_pack", &color, &order)?))
}

/// 按 ARGB 顺序解包。
#[duck_scalar_function(
    overloads_name = "plt_unpack",
    description = "Unpack a 32-bit ARGB integer into an sRGB color (8 bits per channel)",
    example = "SELECT plt_unpack(4294901760)"
)]
fn plt_unpack_1(value: i64) -> DuckOptionResult<PltColor> {
    Ok(Some(cast::unpack("plt_unpack", value, "argb")?))
}

/// 按指定通道顺序解包。
#[duck_scalar_function(
    overloads_name = "plt_unpack",
    description = "Unpack a 32-bit integer in the given channel order into an sRGB color",
    example = "SELECT plt_unpack(4278190335, 'rgba')"
)]
fn plt_unpack_2(value: i64, order: String) -> DuckOptionResult<PltColor> {
    Ok(Some(cast::unpack("plt_unpack", value, &order)?))
}
