# TODO

palette 全部对外能力 → SQL，函数前缀 `plt_`。完成一项勾一项。

- [x] 骨架：改名 `duckfn_palette`、依赖（palette / csscolorparser / colorous / rand）、`plt_color` 类型
- [x] 构造与转换：44 个空间构造器、`plt_convert` / `plt_convert_clamped` / `plt_clamp`、alpha 读写
- [x] 操作族：`plt_mix` / lighten / darken / saturate / desaturate / hue / premultiply
- [x] 混合与合成：11 种 `Blend`、6 种 `Compose`、`plt_blend_equations`
- [x] 色差与对比度：7 种度量、WCAG 相对亮度 / 对比度 / 判定
- [x] 色彩理论、命名色、十六进制与 CSS 解析 / 序列化
- [x] cast / 编码信息 / 白点 / 色适应 / 矩阵
- [x] CAM16 全量（自定义观察条件）、colorous 色板、随机色
- [ ] SQL 测试：官方用例全覆盖 + 边界 / NULL / 错误
- [ ] 文档与收尾：README / 描述缺口 / `just lint` / 提交
