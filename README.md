<div align="center">

# ToolTree

**A free, open-source developer toolbox for macOS, Windows and Linux: everyday dev tools, clipboard history and screenshots.**

Built with Rust and Tauri. Lives in the menu bar / tray and opens from anywhere with a global shortcut. Everything runs locally — no data ever leaves your machine.

[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)
![Platforms](https://img.shields.io/badge/platform-macOS%20%7C%20Windows%20%7C%20Linux-lightgrey)

English · [简体中文](README.zh-CN.md)

[**Download**](https://github.com/ADroidCoder/ToolTree/releases/latest) · [Features](#features) · [Building from source](#building-from-source)

</div>

## Download

Get the latest installer from the [**Releases page**](https://github.com/ADroidCoder/ToolTree/releases/latest).

| Platform | File |
|---|---|
| macOS (Apple Silicon) | `ToolTree_<version>_aarch64.dmg` |
| macOS (Intel) | `ToolTree_<version>_x64.dmg` |
| Windows 10 / 11 (x64) | `ToolTree_<version>_x64-setup.exe` or `ToolTree_<version>_x64_en-US.msi` |
| Linux (x64) | `ToolTree_<version>_amd64.deb` (Ubuntu, Debian), `ToolTree-<version>-1.x86_64.rpm` (Fedora, openSUSE) or `ToolTree_<version>_amd64.AppImage` (other distros — make it executable and run it) |

> [!NOTE]
> The installers are not code-signed yet.
> - **macOS**: if macOS says the app "is damaged" or "can't be opened", run `xattr -cr /Applications/ToolTree.app` once in Terminal, or right-click the app and choose Open.
> - **Windows**: if SmartScreen appears, click **More info → Run anyway**.

## Features

### Launcher

Press `⌥Space` (Windows / Linux: `Ctrl+Alt+Space`) to bring up a search box near the top of the screen:

- **Open apps**: with app icons, pinyin and initials support (`wx` finds 微信, `vsc` finds Visual Studio Code); the ones you use most rank first
- **Find files** by name in your home folder; start with `f ` to search files only. Uses the Spotlight index on macOS, [Everything](https://www.voidtools.com/) on Windows when installed (put `es.exe` on your PATH), and `plocate` / `locate` on Linux, falling back to a direct scan of your home folder
- **Calculator**: type `12*8+3`, `2^10`, `sqrt(2)` or `200*15%` for an instant result; Enter copies it
- **Tools and commands**: open any dev tool; pasted JSON, timestamps and the like suggest the matching tool; take a screenshot or open clipboard history
- Enter opens, `⌘↵` shows in Finder, `Esc` clears and then closes; with an empty box it lists what you used recently

> App icons are currently shown on macOS only; Windows and Linux use a generic icon.

### Developer tools

| Category | Tools |
|---|---|
| Format & convert | JSON (format, minify, escape, sort keys, to YAML, generate TypeScript types), number bases (BigInt), colors (HEX / RGB / HSL / HSV, contrast check) |
| Encode & decode | Base64 (text, files, image preview), URL encode / decode (breaks down the URL and query), JWT decoder (expiry, HMAC signature check) |
| Generate & hash | UUID v4 / v7, ULID, NanoID (with timestamp decoding), hashes (MD5, SHA family, file hashes, HMAC) |
| Text | Text diff (inline highlights), regex tester (common pattern library, replace preview), case conversion and word count |
| Time | Timestamp converter (s, ms, µs, ns, multiple time zones) |

### Clipboard history

- Records copied text, images and files automatically; identical content is kept once
- Search, filter by type, pin items; old items are cleaned up by count and age
- **Content detection**: JSON, timestamps, JWTs, Base64, URLs, colors… are labelled and open in the matching tool with one click
- Pop up the panel with a shortcut and press Enter to paste straight into the previous app
- Skips content that password managers mark as concealed; you can also ignore specific apps

### Screenshots

- Hover to detect windows, click to select a whole window; drag to select any region, drag the edges to resize, nudge with arrow keys
- Annotate: rectangle, ellipse, arrow, pen, text, mosaic and numbered markers, with color and size options; `⌘Z` to undo
- The magnifier shows coordinates and color; press `C` with no selection to copy the color under the cursor
- Enter or double-click to copy, `⌘S` to save, `F3` to pin to the desktop (pins can be dragged, zoomed with the scroll wheel and closed with a double-click)
- Multiple monitors; Retina screens export at full resolution. The screen is frozen while you select, so videos and animations don't get in the way

> On macOS, screenshots need permission under System Settings → Privacy & Security → Screen Recording.

### Keyboard shortcuts

| Shortcut | Action |
|---|---|
| `⌥Space` (Windows / Linux: `Ctrl+Alt+Space`) | Open the launcher: search apps, files and tools, or do quick math |
| `⌘⇧V` (Windows / Linux: `Ctrl+Shift+V`) | Show the clipboard history panel |
| `⌘⇧1` (Windows / Linux: `F1`) | Take a screenshot |
| `⌘K` | Search tools in the main window; pasted content is detected automatically |

Global shortcuts can be changed in Settings. Tool search supports pinyin and initials, e.g. `zz` finds the regex tester and `sjc` finds the timestamp converter.

> On macOS, "paste after selecting" needs permission under System Settings → Privacy & Security → Accessibility.

### App

- **Dark / light / system** appearance and a **Chinese / English interface** (follows the system language by default, switchable in Settings).
- Launch at login; on startup it waits quietly in the menu bar (tray).
- **Automatic updates**: checks GitHub Releases in the background, shows a notice at the bottom of the sidebar when a new version is out, and installs and restarts in one click. You can also check manually in Settings → General. Update packages are signed and verified before installing.

## Building from source

**Requirements**

- [Rust](https://rustup.rs/) 1.85 or later
- [Node.js](https://nodejs.org/) 22 or later
- Linux only: `libwebkit2gtk-4.1-dev`, `libgtk-3-dev`, `librsvg2-dev`, `libayatana-appindicator3-dev`, `libxdo-dev`, plus `libxcb1-dev`, `libxrandr-dev`, `libpipewire-0.3-dev`, `libwayland-dev`, `libegl-dev`, `libgbm-dev` and `libclang-dev` for screenshots (Ubuntu / Debian package names; needs PipeWire 1.0 or later, i.e. Ubuntu 24.04 or newer)
- Windows only: [WebView2](https://developer.microsoft.com/microsoft-edge/webview2/) (included with Windows 10/11) and the Visual Studio C++ build tools

```bash
git clone https://github.com/ADroidCoder/ToolTree.git
cd ToolTree
npm install

npm run tauri dev      # run in development mode
npm run tauri build    # build installers
```

**Tests**

```bash
cargo test -p tooltree-core   # clipboard storage, hashing
npm test                      # content detection, tool conversions, screenshot annotations
npm run check                 # type check
```

**Releases** are built by [GitHub Actions](.github/workflows/build.yml):

1. Set the new version in `src-tauri/tauri.conf.json`, `src-tauri/Cargo.toml` and `package.json` (all three must match; CI checks this).
2. Commit, then push a tag: `git tag v0.2.0 && git push origin v0.2.0`.
3. CI builds installers for Windows, both kinds of Mac and Linux, publishes a Release and attaches `latest.json`, so installed copies are offered the update.

Before the first release, add the updater signing key under the repository's Settings → Secrets and variables → Actions: `TAURI_SIGNING_PRIVATE_KEY` (the contents of the private key file) and `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` (its password). If the key is lost, installed versions can never receive updates again, so keep a backup.

Ordinary pushes only run the tests. Running the workflow by hand from the Actions page builds installers without publishing; download them from that run's Artifacts.

## Adding a tool

Create a folder under `src/tools/` with an `index.ts` (name, category, icon, search keywords) and a `Tool.svelte`. It shows up in the sidebar and search automatically — nothing else needs to change. See `src/tools/radix/` for an example.

If the tool can recognise a kind of clipboard content, add a rule in `src/lib/detect.ts`.

## Project layout

```
crates/tooltree-core/   Clipboard history storage (SQLite + PNG) and hashing; UI-independent and testable on its own
src-tauri/              Tray, global shortcuts, clipboard watcher, screen capture, launcher (apps, file search), windows
src/tools/              One folder per tool, registered automatically
src/components/         Main window UI
src/windows/            Launcher, clipboard pop-up panel, screenshot editor, pinned images
src/lib/shot/           Annotation shapes and drawing (shared by display and export)
tests/                  Frontend tests
```

**Tech stack:** Rust · [Tauri 2](https://tauri.app/) · [Svelte 5](https://svelte.dev/) · [xcap](https://github.com/nashaofu/xcap) · [arboard](https://github.com/1Password/arboard) · SQLite · TypeScript

## Contributing

Issues and pull requests are welcome. Before opening a PR, please make sure `cargo test -p tooltree-core`, `npm test` and `npm run check` all pass.

## License

Licensed under the [Apache License 2.0](LICENSE): free for personal and commercial use, and you may modify and redistribute it. Redistributions must keep the copyright and license notices and state any changes made.
