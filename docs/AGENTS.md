# AGENTS.md —— docs 站约定

这份文件是给改 `docs/` 的人（含 AI agent）看的：可运行 SQL 块里最常踩的坑，以及几条约定。
站点本身的布局、命令与部署见 [`README.md`](./README.md)；仓库级的构建/发版流程见根目录的
[`AGENTS.md`](../AGENTS.md)。

## 可运行 SQL 块

info string 是 JSON 的 `sql` 围栏会变成能就地跑的示例（完整配置见 `README.md` 的
「Runnable SQL blocks」）。本节只记约定，配置项本身不在这里重复。

- **不要在块里写死站点的路径前缀**。Docusaurus 会把 `static/` 复制进每个 locale 的输出，同一个
  `data/x.tsv` 在英文页是 `/<repo>/data/x.tsv`、在中文页是 `/<repo>/zh-Hans/data/x.tsv` ——
  写死前缀的块在一个语言下能跑、另一个语言下404（这正是 `start:zh-Hans` 曾经只有英文能跑通的
  原因）。要读站点里的文件就用 `{{DFK_BASE_URL}}`（见下）。
- **示例数据用真实数据集**，不要自造的玩具数据（如 `[[1,2],[3,4]]`）。
- 演示失败的块写 `"expect":"error"`，它由 `npm test` 双向校验（`test.fail()`）。

### `{{DFK_BASE_URL}}`：数据 URL 必须是绝对 URL，且带当前语言的 baseUrl

DuckDB-Wasm 跑在 base URL 为 `blob:` 的 Worker 里，**任何**相对路径都不会拿页面当基准去解析：
`'data/x.tsv'` 和 `'/<repo>/data/x.tsv'` 都会被当成内存文件系统里的路径，报
`IO Error: No files found that match the pattern`。只有绝对的 `http(s)` URL 才走 HTTP 文件系统，
而 origin 恰恰是构建期替换无法知道的那部分（GitHub Pages、`docusaurus start`、测试的随机端口各不相同）。

kit 提供两个运行时占位符（`sql/placeholders`），都在 `DuckDBRuntime.execute()` 里、SQL 交给
DuckDB 之前展开：

| 占位符 | 展开成 |
| --- | --- |
| `{{DFK_ORIGIN}}` | `window.location.origin`，例如 `https://<owner>.github.io` |
| `{{DFK_BASE_URL}}` | origin **加上当前页面的 baseUrl**，例如 `https://<owner>.github.io/<repo>/` 或 `…/<repo>/zh-Hans/` |

**一律用 `{{DFK_BASE_URL}}`，不要把前缀写死在块里**。页面上的 SQL 框显示的已经是展开后的 URL，
读者看不到占位符；`execute()` 会再展开一次，所以 Reset、手改过的 SQL 与 harness 走的都是同一段文本。

这两个占位符是 `duckfn-docs-kit` **0.7.0** 起提供的（配套的 `baseUrl` 选项也是 0.7.0）。

### 静态资源映射（测试侧）

数据在真实站点上由 Docusaurus 从 `static/<dir>/` 提供（`baseUrl` 由 `DOCS_BASE_URL` 注入，见
`docusaurus.config.ts`），而 `playwright test` 跑的是 kit 起的短命 loopback 服务器，它默认只认
`harness.html` / `harness.js` / `/vendor/*` / `/ext/*`。所以凡是有块要读 `static/` 下的目录，都要在
[`tests/docs.spec.mts`](./tests/docs.spec.mts) 里给 `declareDocsTests` 声明 baseUrl 与资源映射：

```ts
declareDocsTests({
  siteDir: fileURLToPath(new URL('..', import.meta.url)),
  baseUrl: '/<repo>/',
  assets: [{url: '/<repo>/data', dir: 'static/data'}],
});
```

harness 没有 locale、中英文的块都在同一页上跑，所以只挂一个前缀、取英文那个 baseUrl；`baseUrl`
与 `assets` 的前缀必须一致，否则块会404（也可以用 `DFK_BASE_URL` / `DFK_ASSETS` 环境变量，或
CLI 的 `--base-url` / `--asset`）。

新增别的静态数据目录时，在这里再挂一条 `{url, dir}`（`url` 以 `/` 开头并与块里写的路径一致，
`dir` 相对 `siteDir`）。静态资源映射是 `duckfn-docs-kit` **0.6.0** 起提供的；模板的块目前只读
内联数据（`VALUES` / 字符串字面量），用不到这一节——等到有块要读 `static/` 下的文件时再回来照它配。

**验证时记住两条坑**：

- 本地调试用的是 `docs/node_modules/duckfn-docs-kit` 里那一份 kit。若手工替换过它，`npm run build`
  仍可能复用旧的 webpack 缓存（页面报 `unknown key baseUrl` 之类），先 `npm run clear`
  或删掉 `node_modules/.cache` 再构建。
- 站点运行时从 CDN（jsDelivr）取引擎 wasm，**本机没有外网时站内点 Run 会停在「正在初始化
  DuckDB…」**；这不是代码问题。离线验证走 `npm test`：kit 的 harness 用本地引擎与 loopback
  服务器，所有块都在那里跑。

## 展示图像

文档里出现图，只有两条路，别混用：

| 图的来源 | 怎么放 | 例子 |
| --- | --- | --- |
| **扩展自己算出来的**（返回值是 SVG / HTML） | 可运行 SQL 块 + `show` | `{"type":"duckfn","show":"svg"}` |
| **站点自己的静态图**（logo、社交卡片、示意图） | `static/img/` + `useBaseUrl('img/…')` | `useBaseUrl('img/logo.svg')` |

### 块里渲染

`show` 可选 `table`（默认）、`text`、`html`、`iframe`、`svg`：

- **返回值是一份 SVG 文档**（画图类扩展）→ `"show":"svg"`。图画在结果区，可全屏缩放/平移，末尾的
  `Table` 标签留着原始标记。
- **交互式内容**（注入 `<script>`、有自己的坐标系与快捷键）→ `"show":"iframe"`。放进宿主文档会跟页面
  抢快捷键、按图定位的控件会改成按页面定位；iframe 是沙箱（`allow-scripts`、无 `allow-same-origin`），
  才有自己的坐标系。
- 普通结果表用默认的 `table` 就够，写 `"show":"table"` 只是显式一点。

三条不要：

- **不要钉画布尺寸**（规格里的 `width` / `height`）。扩展默认的画布比例是调过的，压成又宽又扁会明显
  变形。
- **不要给 `"show":"svg"` 的块加 `option.height`**：它按 SVG 自身高度自动撑开，写死的数字只会在宽高比
  不同的图上留白、或在窄屏下把图裁掉。唯一需要手调高度的是 `iframe` —— iframe 没有内在高度。
- **不要给「块本身就能画出来的图」再配一张 `<img>` 截图**：截图会过期、会锁死一个主题的配色，而且和
  块里的数据对不上。

### 静态图

- 放 `static/img/`，引用走 `useBaseUrl('img/…')`，**不要写死 `/<repo>/img/…`**：站点发布在 `/<repo>/`
  子路径下，写死的前缀在本地与线上不一致。
- `static/` 会被复制进**每个 locale** 的输出，所以跨语言共用的图放这里一份即可，路径里不要带语言前缀；
  只给某一页用的图也可以跟 `.md` 放一起用相对路径引用，但那样中英两份要各放一份。
- 体积：logo 用 SVG（十几 KB），社交卡片 JPG 几十 KB 量级。不要提交构建产物、整页截图或几百 KB 的
  PNG —— 它们会被打进站点，也会留在 git 历史里。
- `favicon` 与链接预览图（`image`）在 `docusaurus.config.ts` 里；换 logo 时 logo、favicon、社交卡片与
  `src/css/custom.css` 的品牌色一起换（那份注释里写明了「palette 来自 logo.svg，两处一起改」）。

## 中英双语要成对改

站点有两个 locale（英文 + `i18n/zh-Hans/…`）。同一页在 `docs/` 与
`docs/i18n/zh-Hans/docusaurus-plugin-content-docs/current/` 下各有一份，**改了一边就
按同样的相对路径改另一边**：`id` / `slug` / `sidebar_position` 保持一致，页内链接用相对
文件路径，可运行块里的 SQL 是代码、照抄，只翻注释。