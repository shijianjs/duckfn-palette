// SQL 侧的类型定义：
//   - `color.rs`：带空间标签的通用颜色值 `plt_color`
//     （`STRUCT(space VARCHAR, c1 DOUBLE, c2 DOUBLE, c3 DOUBLE, alpha DOUBLE)`），
//     以及 CAM16 完整属性的 `plt_cam16_full`；
//   - `ops.rs`：逐空间操作（mix / lighten / saturate / hue / premultiply）的分派；
//   - `blend.rs`：混合（`Blend`）、合成（`Compose`）与 OpenGL 式 `Equations`；
//   - `difference.rs`：色差（`Ciede2000` / `DeltaE` / `EuclideanDistance` / `HyAb`）与 WCAG 对比度；
//   - `theory.rs`：传统色彩理论（互补 / 邻近 / 三角 / 四方）；
//   - `encoding.rs`：传输函数、白点归属、原色与 RGB↔XYZ 矩阵；
//   - `adaptation.rs`：白点数值、色适应矩阵与方法；
//   - `matrix.rs`：3x3 矩阵的校验与运算；
//   - `cast.rs`：8 位打包/解包（`cast::Packed`）；
//   - `cam16.rs`：CAM16 的自定义观察条件（`Parameters` / `Surround` / `Discounting`）；
//   - `random.rs`：随机色（palette 的 `Standard` 分布 + 种子）；
//   - `colorous.rs`：colorous 的渐变色板与分类色板。
//
// SQL-facing types: the tagged color value `plt_color` plus the full CAM16 struct in `color.rs`,
// per-space operation dispatch in `ops.rs`, blending/composition in `blend.rs`, color difference
// plus WCAG contrast in `difference.rs`, color theory in `theory.rs`, encoding/adaptation/matrix
// helpers in `encoding.rs` / `adaptation.rs` / `matrix.rs`, packed integers in `cast.rs`, custom
// CAM16 viewing conditions in `cam16.rs`, random sampling in `random.rs`, and colorous presets in
// `colorous.rs`.
pub(crate) mod adaptation;
pub(crate) mod blend;
pub(crate) mod cam16;
pub(crate) mod cast;
pub(crate) mod color;
pub(crate) mod colorous;
pub(crate) mod difference;
pub(crate) mod encoding;
pub(crate) mod matrix;
pub(crate) mod ops;
pub(crate) mod random;
pub(crate) mod theory;
