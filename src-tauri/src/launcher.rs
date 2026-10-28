//! 启动器：全局快捷键弹出的搜索框，搜应用、文件和工具，也能直接算数。
//! 这里负责窗口、列出装了的应用、取图标、搜文件、打开；匹配和排序在界面里做。
//!
//! - 应用：macOS 扫描 /Applications 等目录里的 .app，Windows 读开始菜单里的快捷方式，Linux 读 .desktop 文件
//! - 文件：macOS 用 Spotlight（mdfind），不用自己建索引；Windows 装了 Everything 时用它的命令行 es.exe，
//!   Linux 有 plocate / locate 时用它们；都没有时在用户目录里限时遍历

use crate::platform::{self, Previous};
use serde::Serialize;
use std::collections::{HashMap, HashSet};
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime};
use tauri::ipc::Response;
use tauri::{AppHandle, Emitter, Manager, PhysicalPosition, State};

pub const LAUNCHER: &str = "launcher";
/// 应用列表缓存多久，装了新应用最多这么久之后就能搜到
const APPS_TTL: Duration = Duration::from_secs(30);
/// 搜文件时最多看多少个候选，再多的排序也排不到前面
const MAX_CANDIDATES: usize = 600;

fn err(e: impl std::fmt::Display) -> String {
    e.to_string()
}

#[derive(Clone, Serialize)]
pub struct App {
    /// 显示的名字（macOS 上是本地化的名字，比如“系统设置”）
    pub name: String,
    /// 另一个名字（一般是英文文件名），同样能搜到
    pub alt: String,
    pub path: String,
}

#[derive(Serialize)]
pub struct FileHit {
    pub path: String,
    pub name: String,
    pub dir: bool,
}

pub struct LauncherState {
    apps: Mutex<Option<(Instant, Vec<App>)>>,
    /// 图标 PNG。普通文件按扩展名共用一个
    icons: Mutex<HashMap<String, Arc<Vec<u8>>>>,
    /// 打开启动器之前在前台的应用，按 Esc 关掉时切回去
    previous: Mutex<Previous>,
    /// 什么时候打开的。刚打开时的失焦是系统激活应用造成的，不算点了别处
    shown_at: Mutex<Option<Instant>>,
}

impl Default for LauncherState {
    fn default() -> Self {
        LauncherState { apps: Mutex::new(None), icons: Mutex::new(HashMap::new()), previous: Mutex::new(Previous::None), shown_at: Mutex::new(None) }
    }
}

// ---- 窗口 ----

/// 在鼠标所在的屏幕上方居中打开；已经打开就关掉
pub fn toggle(app: &AppHandle) {
    let Some(w) = app.get_webview_window(LAUNCHER) else { return };
    if w.is_visible().unwrap_or(false) {
        hide(app, true);
        return;
    }
    *app.state::<LauncherState>().previous.lock().unwrap() = platform::remember_frontmost();
    match (crate::cursor_monitor(app), w.outer_size()) {
        (Some(m), Ok(size)) => {
            let area = m.work_area();
            let x = area.position.x + (area.size.width as i32 - size.width as i32) / 2;
            let y = area.position.y + (area.size.height as f64 * 0.2) as i32;
            let _ = w.set_position(PhysicalPosition::new(x, y));
        }
        _ => {
            let _ = w.center();
        }
    }
    *app.state::<LauncherState>().shown_at.lock().unwrap() = Some(Instant::now());
    let _ = w.show();
    let _ = w.set_focus();
    let _ = w.emit("launcher-show", ());
}

/// restore：切回打开前的应用（按 Esc 时）。打开了别的应用、点了别处时不用
pub fn hide(app: &AppHandle, restore: bool) {
    if let Some(w) = app.get_webview_window(LAUNCHER)
        && w.is_visible().unwrap_or(false)
    {
        let _ = w.hide();
        if restore {
            platform::activate(*app.state::<LauncherState>().previous.lock().unwrap());
        }
    }
}

/// 失去焦点：点了别处就收起来。
/// 主窗口开着时，系统激活应用后会把焦点还给主窗口，刚打开的启动器马上就失焦了，这时把焦点要回来
pub fn on_blur(app: &AppHandle) {
    let fresh = app.state::<LauncherState>().shown_at.lock().unwrap().take().is_some_and(|t| t.elapsed() < Duration::from_millis(500));
    match app.get_webview_window(LAUNCHER) {
        Some(w) if fresh => {
            let _ = w.set_focus();
        }
        _ => hide(app, false),
    }
}

#[tauri::command]
pub fn launcher_hide(app: AppHandle) {
    hide(&app, true);
}

// ---- 应用 ----

#[tauri::command]
pub async fn launcher_apps(state: State<'_, LauncherState>) -> Result<Vec<App>, String> {
    if let Some((at, list)) = &*state.apps.lock().unwrap()
        && at.elapsed() < APPS_TTL
    {
        return Ok(list.clone());
    }
    let list = tauri::async_runtime::spawn_blocking(scan_apps).await.map_err(err)?;
    *state.apps.lock().unwrap() = Some((Instant::now(), list.clone()));
    Ok(list)
}

#[cfg(target_os = "macos")]
fn scan_apps() -> Vec<App> {
    use objc2_foundation::{NSFileManager, NSString};

    fn walk(dir: &Path, depth: u32, out: &mut Vec<PathBuf>) {
        let Ok(rd) = std::fs::read_dir(dir) else { return };
        for e in rd.flatten() {
            let p = e.path();
            let name = e.file_name();
            if name.to_string_lossy().starts_with('.') {
                continue;
            }
            if p.extension().is_some_and(|x| x == "app") {
                out.push(p);
            } else if depth < 2 && p.is_dir() {
                // /Applications/Utilities、厂商的文件夹里也有应用
                walk(&p, depth + 1, out);
            }
        }
    }

    let home = std::env::var("HOME").unwrap_or_default();
    let mut paths = vec![PathBuf::from("/System/Library/CoreServices/Finder.app")];
    for root in ["/Applications", "/System/Applications", "/System/Library/CoreServices/Applications", &format!("{home}/Applications")] {
        walk(Path::new(root), 0, &mut paths);
    }
    let fm = NSFileManager::defaultManager();
    let mut seen = HashSet::new();
    let mut out = vec![];
    for p in paths {
        let path = p.to_string_lossy().into_owned();
        if !seen.insert(path.clone()) {
            continue;
        }
        let alt = p.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();
        // 本地化的名字，比如 System Settings.app 显示成“系统设置”
        let display = fm.displayNameAtPath(&NSString::from_str(&path)).to_string();
        let name = display.strip_suffix(".app").unwrap_or(&display).to_string();
        out.push(App { name, alt, path });
    }
    out
}

#[cfg(target_os = "windows")]
fn scan_apps() -> Vec<App> {
    fn walk(dir: &Path, depth: u32, out: &mut Vec<PathBuf>) {
        let Ok(rd) = std::fs::read_dir(dir) else { return };
        for e in rd.flatten() {
            let p = e.path();
            if p.is_dir() {
                if depth < 4 {
                    walk(&p, depth + 1, out);
                }
            } else if p.extension().is_some_and(|x| x.eq_ignore_ascii_case("lnk") || x.eq_ignore_ascii_case("url")) {
                out.push(p);
            }
        }
    }

    let mut paths = vec![];
    for var in ["ProgramData", "APPDATA"] {
        if let Ok(base) = std::env::var(var) {
            walk(&Path::new(&base).join(r"Microsoft\Windows\Start Menu\Programs"), 0, &mut paths);
        }
    }
    let mut seen = HashSet::new();
    let mut out = vec![];
    for p in paths {
        let name = p.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();
        let lower = name.to_lowercase();
        // 开始菜单里常有卸载程序、说明文档的快捷方式
        if name.is_empty() || lower.contains("uninstall") || name.contains("卸载") || !seen.insert(lower) {
            continue;
        }
        out.push(App { alt: String::new(), name, path: p.to_string_lossy().into_owned() });
    }
    out
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
fn scan_apps() -> Vec<App> {
    let home = std::env::var("HOME").unwrap_or_default();
    let dirs = [
        "/usr/share/applications".to_string(),
        "/usr/local/share/applications".to_string(),
        "/var/lib/flatpak/exports/share/applications".to_string(),
        "/var/lib/snapd/desktop/applications".to_string(),
        format!("{home}/.local/share/applications"),
        format!("{home}/.local/share/flatpak/exports/share/applications"),
    ];
    // Name[zh_CN] 这样的本地化名字
    let lang = std::env::var("LANG").unwrap_or_default();
    let lang = lang.split('.').next().unwrap_or_default().to_string();
    let short = lang.split('_').next().unwrap_or_default().to_string();
    let mut by_id: HashMap<String, App> = HashMap::new();
    for d in dirs {
        let Ok(rd) = std::fs::read_dir(&d) else { continue };
        for e in rd.flatten() {
            let p = e.path();
            if p.extension().is_none_or(|x| x != "desktop") {
                continue;
            }
            let Ok(text) = std::fs::read_to_string(&p) else { continue };
            let (mut name, mut local, mut local_short) = (String::new(), String::new(), String::new());
            let (mut hidden, mut is_app, mut in_entry) = (false, false, false);
            for line in text.lines() {
                let line = line.trim();
                if line.starts_with('[') {
                    in_entry = line == "[Desktop Entry]";
                    continue;
                }
                if !in_entry {
                    continue;
                }
                let Some((k, v)) = line.split_once('=') else { continue };
                match k.trim() {
                    "Name" => name = v.trim().to_string(),
                    "Type" => is_app = v.trim() == "Application",
                    "NoDisplay" | "Hidden" if v.trim() == "true" => hidden = true,
                    k if !lang.is_empty() && k == format!("Name[{lang}]") => local = v.trim().to_string(),
                    k if !short.is_empty() && k == format!("Name[{short}]") => local_short = v.trim().to_string(),
                    _ => {}
                }
            }
            if hidden || !is_app || name.is_empty() {
                continue;
            }
            let shown = [local, local_short].into_iter().find(|s| !s.is_empty()).unwrap_or_else(|| name.clone());
            let id = e.file_name().to_string_lossy().into_owned();
            // 用户目录里的同名文件覆盖系统的，按目录顺序后面的优先
            by_id.insert(id, App { alt: if shown == name { String::new() } else { name }, name: shown, path: p.to_string_lossy().into_owned() });
        }
    }
    by_id.into_values().collect()
}

// ---- 图标 ----

/// 普通文件的图标只跟类型有关，按扩展名缓存；应用、文件夹、快捷方式各有各的图标
fn icon_key(path: &str) -> String {
    let p = Path::new(path);
    let ext = p.extension().map(|x| x.to_string_lossy().to_lowercase()).unwrap_or_default();
    if p.is_file() && !["app", "exe", "lnk", "url", "desktop"].contains(&ext.as_str()) { format!("ext:{ext}") } else { path.to_string() }
}

/// 应用、文件的图标，64 像素的 PNG（界面里按 32 显示，Retina 屏上也清楚）
#[tauri::command]
pub async fn launcher_icon(state: State<'_, LauncherState>, path: String) -> Result<Response, String> {
    let key = icon_key(&path);
    if let Some(png) = state.icons.lock().unwrap().get(&key) {
        return Ok(Response::new(png.to_vec()));
    }
    let png = tauri::async_runtime::spawn_blocking(move || icon_png(&path)).await.map_err(err)??;
    state.icons.lock().unwrap().insert(key, Arc::new(png.clone()));
    Ok(Response::new(png))
}

#[cfg(target_os = "macos")]
fn icon_png(path: &str) -> Result<Vec<u8>, String> {
    use objc2::AllocAnyThread;
    use objc2::rc::autoreleasepool;
    use objc2_app_kit::{NSBitmapImageFileType, NSBitmapImageRep, NSCompositingOperation, NSDeviceRGBColorSpace, NSGraphicsContext, NSWorkspace};
    use objc2_foundation::{NSDictionary, NSPoint, NSRect, NSSize, NSString};

    const PX: isize = 64;
    // 画图用的是线程自己的当前上下文，但图标缓存这些系统内部的东西不保证能并发，一次画一个
    static LOCK: Mutex<()> = Mutex::new(());
    let _guard = LOCK.lock().unwrap();
    autoreleasepool(|_| {
        let img = NSWorkspace::sharedWorkspace().iconForFile(&NSString::from_str(path));
        // SAFETY: 参数是一张 64×64、每像素 4 个 8 位通道的 RGBA 位图，数据由系统分配
        let rep = unsafe {
            NSBitmapImageRep::initWithBitmapDataPlanes_pixelsWide_pixelsHigh_bitsPerSample_samplesPerPixel_hasAlpha_isPlanar_colorSpaceName_bytesPerRow_bitsPerPixel(
                NSBitmapImageRep::alloc(),
                std::ptr::null_mut(),
                PX,
                PX,
                8,
                4,
                true,
                false,
                NSDeviceRGBColorSpace,
                0,
                0,
            )
        }
        .ok_or("创建位图失败")?;
        let ctx = NSGraphicsContext::graphicsContextWithBitmapImageRep(&rep).ok_or("创建画布失败")?;
        NSGraphicsContext::saveGraphicsState_class();
        NSGraphicsContext::setCurrentContext(Some(&ctx));
        let rect = NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(PX as f64, PX as f64));
        img.drawInRect_fromRect_operation_fraction(rect, NSRect::ZERO, NSCompositingOperation::Copy, 1.0);
        ctx.flushGraphics();
        NSGraphicsContext::restoreGraphicsState_class();
        // SAFETY: 空的属性字典，用默认的 PNG 参数
        let data = unsafe { rep.representationUsingType_properties(NSBitmapImageFileType::PNG, &NSDictionary::new()) }.ok_or("转成 PNG 失败")?;
        Ok(data.to_vec())
    })
}

#[cfg(not(target_os = "macos"))]
fn icon_png(_: &str) -> Result<Vec<u8>, String> {
    Err("这个系统上还不支持取图标".into())
}

// ---- 文件 ----

/// 按文件名搜文件。query 按空格分成几个词，文件名要包含所有的词
#[tauri::command]
pub async fn launcher_files(query: String, limit: usize) -> Result<Vec<FileHit>, String> {
    let words: Vec<String> = query.split_whitespace().map(str::to_lowercase).collect();
    if words.is_empty() {
        return Ok(vec![]);
    }
    tauri::async_runtime::spawn_blocking(move || {
        let home = home_dir();
        let found = candidates(&words, &home);
        rank(found, &words, &home, limit.clamp(1, 100))
    })
    .await
    .map_err(err)
}

fn home_dir() -> PathBuf {
    std::env::var_os(if cfg!(windows) { "USERPROFILE" } else { "HOME" }).map(PathBuf::from).unwrap_or_default()
}

/// 读命令输出的每一行当路径，够数了就结束进程
fn run_lines(mut cmd: Command, max: usize) -> Option<Vec<PathBuf>> {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        // 不弹出黑色的命令行窗口
        cmd.creation_flags(0x0800_0000);
    }
    let mut child = cmd.stdout(Stdio::piped()).stderr(Stdio::null()).spawn().ok()?;
    let out = child.stdout.take()?;
    let mut lines = vec![];
    for line in BufReader::new(out).lines() {
        let Ok(line) = line else { break };
        if !line.is_empty() {
            lines.push(PathBuf::from(line));
        }
        if lines.len() >= max {
            break;
        }
    }
    let _ = child.kill();
    let _ = child.wait();
    Some(lines)
}

#[cfg(target_os = "macos")]
fn candidates(words: &[String], home: &Path) -> Vec<PathBuf> {
    // Spotlight 查询语法：c 不区分大小写，d 不区分重音
    let quote = |w: &str| w.replace('\\', "\\\\").replace('"', "\\\"").replace('*', "\\*");
    let q = words.iter().map(|w| format!("kMDItemFSName == \"*{}*\"cd", quote(w))).collect::<Vec<_>>().join(" && ");
    let mut cmd = Command::new("mdfind");
    // 只搜用户目录：系统目录里同名的东西太多，应用已经单独列出来了
    cmd.arg("-onlyin").arg(home).arg(q);
    // 关了 Spotlight 索引（或者 mdfind 用不了）时一个都搜不到，这时自己找
    match run_lines(cmd, MAX_CANDIDATES) {
        Some(list) if !list.is_empty() => list,
        _ => walk_home(words, home),
    }
}

#[cfg(target_os = "windows")]
fn candidates(words: &[String], home: &Path) -> Vec<PathBuf> {
    // Everything 的命令行工具，装了 Everything 并把 es.exe 放进 PATH 才有
    let mut cmd = Command::new("es");
    cmd.arg("-n").arg(MAX_CANDIDATES.to_string()).args(words);
    match run_lines(cmd, MAX_CANDIDATES) {
        Some(list) => list,
        None => walk_home(words, home),
    }
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
fn candidates(words: &[String], home: &Path) -> Vec<PathBuf> {
    // locate 匹配的是整个路径，-b 只看文件名；只能按一个词搜，剩下的词在 rank 里筛
    for bin in ["plocate", "locate"] {
        let mut cmd = Command::new(bin);
        cmd.args(["-i", "-b", "-l", "5000", "--"]).arg(&words[0]);
        if let Some(list) = run_lines(cmd, 5000) {
            return list;
        }
    }
    walk_home(words, home)
}

/// 没有现成的索引时，在用户目录里找，最多花半秒
fn walk_home(words: &[String], home: &Path) -> Vec<PathBuf> {
    const SKIP: [&str; 8] = ["node_modules", "target", "Library", "AppData", "__pycache__", "venv", "Pods", "build"];
    let deadline = Instant::now() + Duration::from_millis(500);
    let mut out = vec![];
    let mut queue = std::collections::VecDeque::from([(home.to_path_buf(), 0u32)]);
    while let Some((dir, depth)) = queue.pop_front() {
        if Instant::now() > deadline || out.len() >= MAX_CANDIDATES {
            break;
        }
        let Ok(rd) = std::fs::read_dir(&dir) else { continue };
        for e in rd.flatten() {
            let name = e.file_name().to_string_lossy().into_owned();
            if name.starts_with('.') {
                continue;
            }
            let lower = name.to_lowercase();
            if words.iter().all(|w| lower.contains(w.as_str())) {
                out.push(e.path());
            }
            if depth < 6 && e.file_type().is_ok_and(|t| t.is_dir()) && !SKIP.contains(&name.as_str()) {
                queue.push_back((e.path(), depth + 1));
            }
        }
    }
    out
}

/// 隐藏目录、依赖目录、应用包内部的文件，基本不会是要找的
fn is_noise(p: &Path, home: &Path) -> bool {
    let rel = p.strip_prefix(home).unwrap_or(p);
    let mut parts = rel.components().map(|c| c.as_os_str().to_string_lossy());
    let first = parts.next().unwrap_or_default();
    if first == "Library" || first == "AppData" {
        // iCloud 云盘在 ~/Library/Mobile Documents 里
        return !rel.starts_with("Library/Mobile Documents");
    }
    let s = p.to_string_lossy();
    rel.components().any(|c| c.as_os_str().to_string_lossy().starts_with('.'))
        || s.contains("/node_modules/")
        || s.contains("\\node_modules\\")
        || s.contains(".app/")
        || s.starts_with("/proc/")
        || s.starts_with("/sys/")
}

fn rank(found: Vec<PathBuf>, words: &[String], home: &Path, limit: usize) -> Vec<FileHit> {
    let joined = words.join(" ");
    let now = SystemTime::now();
    let mut scored: Vec<(i32, FileHit)> = found
        .into_iter()
        .filter(|p| !is_noise(p, home))
        .filter_map(|p| {
            let name = p.file_name()?.to_string_lossy().into_owned();
            let lower = name.to_lowercase();
            if !words.iter().all(|w| lower.contains(w.as_str())) {
                return None;
            }
            let meta = std::fs::metadata(&p).ok()?;
            let stem = p.file_stem().map(|s| s.to_string_lossy().to_lowercase()).unwrap_or_default();
            let mut s = if lower == joined || stem == joined {
                100
            } else if lower.starts_with(&words[0]) {
                80
            } else {
                50
            };
            let rel = p.strip_prefix(home).ok();
            if let Some(rel) = rel {
                if ["Desktop", "Documents", "Downloads"].iter().any(|d| rel.starts_with(d)) {
                    s += 10;
                }
                // 越深的越不像要找的
                s -= rel.components().count() as i32 * 2;
            } else {
                s -= 20;
            }
            if meta.is_dir() {
                s += 3;
            }
            if let Some(age) = meta.modified().ok().and_then(|m| now.duration_since(m).ok()) {
                if age < Duration::from_secs(7 * 86400) {
                    s += 8;
                } else if age < Duration::from_secs(30 * 86400) {
                    s += 4;
                }
            }
            Some((s, FileHit { path: p.to_string_lossy().into_owned(), name, dir: meta.is_dir() }))
        })
        .collect();
    scored.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| a.1.path.len().cmp(&b.1.path.len())));
    scored.truncate(limit);
    scored.into_iter().map(|(_, h)| h).collect()
}

// ---- 打开 ----

/// 打开应用、文件；reveal 为 true 时在访达 / 资源管理器里显示
#[tauri::command]
pub fn launcher_open(app: AppHandle, path: String, reveal: bool) -> Result<(), String> {
    hide(&app, false);
    if reveal { reveal_path(&path) } else { open_path(&path) }
}

fn spawn(mut cmd: Command) -> Result<(), String> {
    cmd.stdout(Stdio::null()).stderr(Stdio::null()).spawn().map(|_| ()).map_err(err)
}

#[cfg(target_os = "macos")]
fn open_path(path: &str) -> Result<(), String> {
    let mut c = Command::new("open");
    c.arg(path);
    spawn(c)
}

#[cfg(target_os = "macos")]
fn reveal_path(path: &str) -> Result<(), String> {
    let mut c = Command::new("open");
    c.arg("-R").arg(path);
    spawn(c)
}

#[cfg(target_os = "windows")]
fn open_path(path: &str) -> Result<(), String> {
    // explorer 会用默认程序打开文件、运行快捷方式
    let mut c = Command::new("explorer");
    c.arg(path);
    spawn(c)
}

#[cfg(target_os = "windows")]
fn reveal_path(path: &str) -> Result<(), String> {
    use std::os::windows::process::CommandExt;
    // /select, 后面的路径要自己加引号，交给标准库会把整个参数包起来，explorer 不认
    let mut c = Command::new("explorer");
    c.raw_arg(format!("/select,\"{path}\""));
    spawn(c)
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
fn open_path(path: &str) -> Result<(), String> {
    if path.ends_with(".desktop") {
        let mut c = Command::new("gio");
        c.arg("launch").arg(path);
        if spawn(c).is_ok() {
            return Ok(());
        }
        let id = Path::new(path).file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();
        let mut c = Command::new("gtk-launch");
        c.arg(id);
        return spawn(c);
    }
    let mut c = Command::new("xdg-open");
    c.arg(path);
    spawn(c)
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
fn reveal_path(path: &str) -> Result<(), String> {
    let dir = Path::new(path).parent().map(|p| p.to_string_lossy().into_owned()).unwrap_or_default();
    let mut c = Command::new("xdg-open");
    c.arg(dir);
    spawn(c)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rank_prefers_exact_and_shallow() {
        let home = PathBuf::from("/nonexistent-home");
        // rank 要读文件信息，用真实存在的临时文件
        let dir = std::env::temp_dir().join(format!("tooltree-rank-{}", std::process::id()));
        std::fs::create_dir_all(dir.join("a/b/c")).unwrap();
        for p in ["report.pdf", "a/b/c/report.pdf", "old-report-2020.pdf", "unrelated.txt"] {
            std::fs::write(dir.join(p), b"x").unwrap();
        }
        let found = ["a/b/c/report.pdf", "old-report-2020.pdf", "report.pdf", "unrelated.txt"].iter().map(|p| dir.join(p)).collect();
        let hits = rank(found, &["report".into()], &home, 10);
        std::fs::remove_dir_all(&dir).unwrap();
        let names: Vec<_> = hits.iter().map(|h| h.path.strip_prefix(dir.to_str().unwrap()).unwrap().to_string()).collect();
        assert_eq!(names.len(), 3, "不包含关键词的文件要被筛掉：{names:?}");
        assert!(names[0].ends_with("/report.pdf") && !names[0].contains("/a/"), "{names:?}");
        assert!(names.iter().position(|n| n.contains("old-report")) > Some(0));
    }

    #[test]
    fn noise_is_filtered() {
        let home = Path::new("/Users/me");
        assert!(is_noise(Path::new("/Users/me/.cache/x.txt"), home));
        assert!(is_noise(Path::new("/Users/me/Library/Caches/x.txt"), home));
        assert!(is_noise(Path::new("/Users/me/code/app/node_modules/x/index.js"), home));
        assert!(!is_noise(Path::new("/Users/me/Library/Mobile Documents/com~apple~CloudDocs/a.pages"), home));
        assert!(!is_noise(Path::new("/Users/me/Documents/a.pdf"), home));
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn mac_apps_and_icons() {
        let apps = scan_apps();
        assert!(apps.len() > 20, "应该能找到不少应用：{}", apps.len());
        let finder = apps.iter().find(|a| a.alt == "Finder").expect("Finder 要在列表里");
        assert!(!finder.name.is_empty());
        assert!(apps.iter().any(|a| a.alt == "Safari"));
        let png = icon_png(&finder.path).unwrap();
        assert_eq!(&png[1..4], b"PNG");
        let img = image::load_from_memory(&png).unwrap();
        assert_eq!((img.width(), img.height()), (64, 64));
        // 图标不能是全透明的
        assert!(img.to_rgba8().pixels().any(|p| p[3] > 0));
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn mac_file_search() {
        let home = home_dir();
        let dir = home.join("Downloads");
        let name = format!("tooltree-search-test-{}.txt", std::process::id());
        let path = dir.join(&name);
        std::fs::write(&path, b"x").unwrap();
        // Spotlight 建索引要一点时间；没开索引时走自己遍历，也要能搜到
        let mut ok = false;
        for _ in 0..20 {
            std::thread::sleep(Duration::from_millis(500));
            let hits = rank(candidates(&["tooltree-search-test".into()], &home), &["tooltree-search-test".into()], &home, 10);
            if hits.iter().any(|h| h.name == name) {
                ok = true;
                break;
            }
        }
        std::fs::remove_file(&path).unwrap();
        assert!(ok, "Spotlight 没有搜到刚建的文件");
    }
}
