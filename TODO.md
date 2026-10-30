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
- [x] SQL 测试：15 份 `.test` 覆盖上游断言 + 边界 / NULL / 错误，`just test` 全绿
- [x] 文档：README（中英）、用户指南（中英）、开发指南（中英）、首页、`function_descriptions.csv`
- [x] 函数页拆分：`functions/` 下六个 generated-index 子目录、13 页中英全文，覆盖全部 145 个函数
- [x] 首页与开篇文案：点明包装 palette / csscolorparser / colorous 三个 Rust 库
- [x] 转换暴力穷举：40 个可逆空间两两往返（1600 次）纳入 `convert.test`
- [x] 社区扩展注册草稿：`description.yml` 占位符全部替换、CSV 已重新生成
- [x] 收尾：`just lint` 无警告、`just test` 全绿、已提交

剩下的都是发版动作，不是功能缺口：

- [ ] 发版：`Cargo.toml` 已是 `0.1.0` 且没有 tag，直接 `just release_tag 0.1.0` → 等 CI 出 Release，
      再 `just release_dev 0.1.1-dev.0`
- [ ] 发版后把 `community-extension/description.yml` 的 `repo.ref` 换成发布提交的 SHA，再向
      duckdb/community-extensions 提 PR
