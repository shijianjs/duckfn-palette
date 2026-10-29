// ============================================================================
// 注册到 DuckDB 的函数总入口。
//
// **首要规则：palette 的全部对外能力都要有 SQL 出口，不允许缺失。** 目录与 palette 的模块树
// 同构，逐条核对「palette 有什么 ↔ 这里暴露了什么」：
//
//   palette                          duckfn_palette
//   src/rgb.rs, src/encoding/*.rs    functions/convert.rs（44 个空间构造器 + 转换）、
//                                    functions/encoding.rs（编码信息/传输函数）
//   src/convert.rs                   functions/convert.rs（plt_convert / plt_to_*）
//   src/alpha/*.rs                   functions/convert.rs（alpha 读写）、functions/blend.rs（premultiply）
//   src/lib.rs（Mix/Lighten/Darken/
//     Saturate/Desaturate/Clamp/…）  functions/manipulate.rs
//   src/hues.rs                      functions/manipulate.rs（hue 读写/位移）
//   src/blend/*.rs                   functions/blend.rs（11 种 blend + 6 种 compose + Equations）
//   src/color_difference.rs          functions/difference.rs
//   src/relative_contrast.rs         functions/difference.rs
//   src/color_theory.rs              functions/theory.rs
//   src/named.rs                     functions/named.rs + functions/css.rs
//   src/cast/*.rs                    functions/cast.rs
//   src/chromatic_adaptation.rs      functions/adaptation.rs
//   src/white_point.rs               functions/adaptation.rs
//   src/lms/*.rs                     functions/convert.rs（lms 空间）+ functions/adaptation.rs
//   src/cam16/*.rs                   functions/convert.rs（默认条件）+ functions/cam16.rs（显式条件）
//   src/matrix.rs                    functions/matrix.rs
//   src/serde.rs                     functions/cast.rs（数组/整数）+ functions/css.rs（字符串）
//   src/random_sampling/*.rs         functions/random.rs
//   外部补充：csscolorparser           functions/css.rs
//   外部补充：colorous                 functions/colorous.rs
//
// 有意不包装的（见根 AGENTS 的「不包装清单」约定，理由写在这里）：
//   1. SIMD / `wide`、`bytemuck` Pod、no_std / libm 路径：SQL 标量没有向量通道，也不暴露内存布局；
//   2. 泛型/类型级机制（白点、RGB 标准、LMS 矩阵、`BoolMask`、trait 泛型本身）：擦除成空间名、
//      白点名与字符串参数；palette 的 148 个命名色常量改由 `plt_named` 查表；
//   3. 就地转换 guard（`FromColorMut*`）与 `TryFromColor` 的 `OutOfBounds` 值：SQL 没有可变借用的
//      生命周期；「转不过去」用 `plt_convert` 的未裁剪值 + `plt_clamp` 组合，或 `plt_is_within_bounds`
//      判断；
//   4. `checked_*` 之类的错误路径孪生：SQL 侧统一「算不出 → NULL / 参数非法 → 查询错误」；
//   5. 容器/迭代器辅助方法（hue 的 push/pop/drain、cast 的 slice/Vec 跨类型整体转换）：SQL 的
//      LIST 已经是值语义，`plt_components` / `plt_from_components` 覆盖单值场景。
//
// 三态契约（所有函数统一）：
//   - 输入 SQL NULL → 该行短路成 NULL（参数写成非 Option 即可）；
//   - 上游「没有结果」（不认识的名字、色差无定义）→ NULL；
//   - 参数非法（未知空间名、未知模式、LIST 长度不对、alpha 越界）→ 查询错误，消息以函数名开头；
//   - ±∞ 与超界浮点是合法结果，原样返回（要不要裁剪由 `plt_clamp` 决定）。
//
// FIRST RULE: every public palette capability gets a SQL surface. The tree mirrors palette's
// modules (table above). Deliberately unwrapped: SIMD/bytemuck/no_std plumbing, type-level generics
// (erased into space/white-point names), in-place conversion guards, `checked_*` twins, and
// container/iterator helpers that SQL's value-semantic LIST already covers. The three-state
// contract is spelled out above.
// ============================================================================

pub(crate) mod adaptation;
pub(crate) mod blend;
pub(crate) mod cast;
pub(crate) mod convert;
pub(crate) mod css;
pub(crate) mod difference;
pub(crate) mod encoding;
pub(crate) mod manipulate;
pub(crate) mod matrix;
pub(crate) mod named;
pub(crate) mod theory;

use duckfn::duck_error;
use quack_rs::error::ExtensionError;

/// 参数非法的统一错误构造：消息以函数名开头，用户报错时能自解释。
///
/// Uniform "invalid argument" error; the message starts with the function name.
pub(crate) fn err(message: impl Into<String>) -> ExtensionError {
    duck_error(message)
}
