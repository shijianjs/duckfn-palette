// SQL 侧的类型定义：
//   - `color.rs`：带空间标签的通用颜色值 `plt_color`
//     （`STRUCT(space VARCHAR, c1 DOUBLE, c2 DOUBLE, c3 DOUBLE, alpha DOUBLE)`），
//     以及 CAM16 完整属性的 `plt_cam16_full`；
//   - `ops.rs`：逐空间操作（mix / lighten / saturate / hue / premultiply）的分派；
//   - `blend.rs`：混合（`Blend`）、合成（`Compose`）与 OpenGL 式 `Equations`；
//   - `difference.rs`：色差（`Ciede2000` / `DeltaE` / `EuclideanDistance` / `HyAb`）与 WCAG 对比度；
//   - `theory.rs`：传统色彩理论（互补 / 邻近 / 三角 / 四方）。
//
// SQL-facing types: the tagged color value `plt_color` plus the full CAM16 struct in `color.rs`,
// the per-space operation dispatch in `ops.rs`, blending/composition in `blend.rs`, color
// difference plus WCAG contrast in `difference.rs`, and color theory in `theory.rs`.
pub(crate) mod blend;
pub(crate) mod color;
pub(crate) mod difference;
pub(crate) mod ops;
pub(crate) mod theory;
