<div align="center">

# ToolTree

**免费、开源的开发者工具箱：常用开发工具 + 剪贴板历史 + 截图，支持 macOS、Windows 和 Linux。**

基于 Rust 和 Tauri 构建，常驻托盘，全局快捷键随时唤起。所有处理都在本地完成，不上传任何数据。

[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)
![Platforms](https://img.shields.io/badge/platform-macOS%20%7C%20Windows%20%7C%20Linux-lightgrey)

[English](README.md) · 简体中文

[**下载**](https://github.com/ADroidCoder/ToolTree/releases/latest) · [功能](#功能) · [从源码构建](#从源码构建)

</div>

## 下载

在 [**Releases 页面**](https://github.com/ADroidCoder/ToolTree/releases/latest) 下载最新安装包。

| 平台 | 文件 |
|---|---|
| macOS（Apple 芯片） | `ToolTree_<版本>_aarch64.dmg` |
| macOS（Intel） | `ToolTree_<版本>_x64.dmg` |
| Windows 10 / 11（x64） | `ToolTree_<版本>_x64-setup.exe` 或 `ToolTree_<版本>_x64_en-US.msi` |
| Linux（x64） | `ToolTree_<版本>_amd64.deb`（Ubuntu、Debian）、`ToolTree-<版本>-1.x86_64.rpm`（Fedora、openSUSE）或 `ToolTree_<版本>_amd64.AppImage`（其他发行版，加上执行权限直接运行） |

> [!NOTE]
> 安装包暂时没有代码签名。
> - **macOS**：如果提示“已损坏”或“无法打开”，在终端运行一次 `xattr -cr /Applications/ToolTree.app`，或者右键点应用选择“打开”。
> - **Windows**：如果出现 SmartScreen 提示，点“更多信息 → 仍要运行”。

## 功能

### 开发工具

| 分类 | 工具 |
|---|---|
| 格式与转换 | JSON（格式化、压缩、转义、排序、转 YAML、生成 TypeScript 类型）、进制转换（BigInt）、颜色转换（HEX / RGB / HSL / HSV、对比度检查） |
| 编码解码 | Base64（文本、文件、图片预览）、URL 编解码（拆解网址和参数）、JWT 解析（过期时间、HMAC 签名校验） |
| 生成与加密 | UUID v4 / v7、ULID、NanoID（含时间解析），哈希（MD5、SHA 系列、文件哈希、HMAC） |
| 文本 | 文本对比（行内差异高亮）、正则测试（常用正则库、替换预览）、命名转换和字数统计 |
| 时间 | 时间戳转换（秒、毫秒、微秒、纳秒，多时区） |

### 剪贴板历史

- 自动记录复制的文本、图片和文件，相同内容只保留一条
- 搜索、按类型筛选、置顶；按条数和天数自动清理
- **自动识别内容**：复制的是 JSON、时间戳、JWT、Base64、网址、颜色……会标出类型，一键用对应工具打开
- 快捷键弹出面板，回车直接粘贴到之前的应用
- 不记录密码管理器标记为隐藏的内容，可以设置忽略的应用

### 截图

- 悬停自动识别窗口，单击选中整个窗口；拖动自由框选，拖动边角调整大小，方向键微调
- 标注：矩形、椭圆、箭头、画笔、文字、马赛克、序号，可选颜色和粗细，`⌘Z` 撤销
- 放大镜显示坐标和颜色，没选区域时按 `C` 复制鼠标位置的颜色
- 回车或双击复制，`⌘S` 保存，`F3` 钉在桌面上（贴图可以拖动、滚轮缩放、双击关闭）
- 多显示器、Retina 屏按原始分辨率导出；框选时画面是静止的，视频和动画不会影响结果

> macOS 上截图需要在 系统设置 → 隐私与安全性 → 屏幕录制 里允许 ToolTree。

### 快捷键

| 快捷键 | 作用 |
|---|---|
| `⌥Space`（Windows / Linux：`Ctrl+Alt+Space`） | 在任何地方打开 ToolTree 并搜索工具 |
| `⌘⇧V`（Windows / Linux：`Ctrl+Shift+V`） | 弹出剪贴板历史面板 |
| `⌘⇧1`（Windows / Linux：`F1`） | 截图 |
| `⌘K` | 在主窗口里搜索工具，粘贴内容可以自动识别 |

全局快捷键可以在设置里修改。工具搜索支持拼音和首字母，比如 `zz` 找到正则测试、`sjc` 找到时间戳。

> macOS 上“选中后直接粘贴”需要在 系统设置 → 隐私与安全性 → 辅助功能 里允许 ToolTree。

### 应用

- **深色 / 浅色 / 跟随系统**三种外观，**中英文界面**（默认跟随系统语言，可以在设置里切换）。
- 开机启动，启动后只在菜单栏（托盘）里待命。
- **自动更新**：后台检查 GitHub Release，有新版本时在侧边栏底部提示，一键安装并重启；也可以在 设置 → 通用 里手动检查。更新包带签名，安装前会校验。

## 从源码构建

**环境要求**

- [Rust](https://rustup.rs/) 1.85 或更新
- [Node.js](https://nodejs.org/) 22 或更新
- 仅 Linux：`libwebkit2gtk-4.1-dev`、`libgtk-3-dev`、`librsvg2-dev`、`libayatana-appindicator3-dev`、`libxdo-dev`，以及截图要的 `libxcb1-dev`、`libxrandr-dev`、`libpipewire-0.3-dev`、`libwayland-dev`、`libegl-dev`、`libgbm-dev`、`libclang-dev`（Ubuntu / Debian 的包名）
- 仅 Windows：[WebView2](https://developer.microsoft.com/microsoft-edge/webview2/)（Windows 10/11 一般自带）和 Visual Studio C++ 生成工具

```bash
git clone https://github.com/ADroidCoder/ToolTree.git
cd ToolTree
npm install

npm run tauri dev      # 开发模式运行
npm run tauri build    # 打包安装包
```

**测试**

```bash
cargo test -p tooltree-core   # 剪贴板存储、哈希
npm test                      # 内容识别、各工具的转换逻辑、截图标注
npm run check                 # 类型检查
```

**发布新版本**由 [GitHub Actions](.github/workflows/build.yml) 完成：

1. 把 `src-tauri/tauri.conf.json`、`src-tauri/Cargo.toml`、`package.json` 里的版本号改成新版本（三处必须一致，CI 会检查）。
2. 提交后推送标签：`git tag v0.2.0 && git push origin v0.2.0`。
3. CI 打出 Windows、两种 Mac 和 Linux 的安装包，发布 Release，并附上 `latest.json`，已安装的用户会收到更新提示。

第一次发版前，在仓库 Settings → Secrets and variables → Actions 里添加自动更新的签名私钥：`TAURI_SIGNING_PRIVATE_KEY`（私钥文件的内容）和 `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`（私钥密码）。私钥一旦丢失，已经安装的版本就再也收不到更新，请务必备份。

平时推送代码只跑测试；在 Actions 页面手动运行可以只打包不发布，安装包在这次运行的 Artifacts 里下载。

## 新增一个工具

在 `src/tools/` 下建一个目录，放 `index.ts`（名字、分类、图标、搜索关键词）和 `Tool.svelte`，会自动出现在侧边栏和搜索里，不需要改其他地方。参考 `src/tools/radix/`。

如果这个工具能识别某种剪贴板内容，在 `src/lib/detect.ts` 里加一条规则。

## 项目结构

```
crates/tooltree-core/   剪贴板历史存储（SQLite + PNG）、哈希，和界面无关，可以单独测试
src-tauri/              托盘、全局快捷键、剪贴板监听、截屏、窗口
src/tools/              每个工具一个目录，自动注册
src/components/         主窗口界面
src/windows/            剪贴板弹出面板、截图编辑、贴图
src/lib/shot/           截图标注的形状和绘制（显示和导出共用）
tests/                  前端测试
```

**技术栈：** Rust · [Tauri 2](https://tauri.app/) · [Svelte 5](https://svelte.dev/) · [xcap](https://github.com/nashaofu/xcap) · [arboard](https://github.com/1Password/arboard) · SQLite · TypeScript

## 参与贡献

欢迎提 Issue 和 Pull Request。提交 PR 前请确认 `cargo test -p tooltree-core`、`npm test` 和 `npm run check` 都能通过。

## 开源协议

本项目基于 [Apache License 2.0](LICENSE) 开源：可以免费用于个人和商业用途，可以修改和再发布；再发布时需要保留版权和协议声明，并注明修改过的地方。
