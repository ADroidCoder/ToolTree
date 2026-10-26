//! 截图：先把每个屏幕截下来，再在每个屏幕上盖一个全屏窗口显示截到的画面，
//! 用户在上面框选、标注，完成后把结果（RGBA 像素）发回来复制、保存或者钉在桌面上。
//!
//! 盖在上面的窗口显示的是截好的静态画面，所以框选时屏幕内容变化（视频、动画）不会影响结果。
//! 这些窗口第一次截图时创建，之后隐藏起来重复用，第二次截图就不用等网页加载了。

use crate::platform;
use image::RgbaImage;
use serde::Serialize;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use tauri::ipc::{InvokeBody, Request, Response};
use tauri::{AppHandle, Emitter, Manager, State, WebviewUrl, WebviewWindow, WebviewWindowBuilder};

/// 截图窗口的标签前缀，后面跟屏幕序号
const OVERLAY: &str = "shot-";
const PIN: &str = "pin-";

/// 窗口吸附用的矩形，相对所在屏幕左上角，单位是 CSS 像素
#[derive(Clone, Copy, Debug, Serialize)]
pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
}

/// 一个屏幕的截图
struct Screen {
    /// 屏幕在桌面上的位置和大小。macOS 上是逻辑点，其他系统是物理像素（和 xcap、窗口坐标的单位一致）
    x: i32,
    y: i32,
    w: u32,
    h: u32,
    /// 上面那个单位换算成 CSS 像素的比例：macOS 上是 1，其他系统是 1 / 缩放比例
    to_css: f64,
    image: RgbaImage,
    windows: Vec<Rect>,
}

/// 截图窗口加载后来取的信息
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ShotInfo {
    /// 每次截图加一，网页据此判断是不是新的一次
    id: u32,
    width: f64,
    height: f64,
    windows: Vec<Rect>,
    /// 鼠标是不是在这个屏幕上：是的话这个窗口获得焦点
    active: bool,
}

/// 钉在桌面上的图
struct Pinned {
    image: RgbaImage,
}

#[derive(Default)]
pub struct ShotState {
    busy: AtomicBool,
    seq: AtomicU32,
    screens: Mutex<Vec<Screen>>,
    /// 截图前在前台的应用，完成后切回去
    previous: Mutex<Option<platform::Previous>>,
    /// 鼠标所在的屏幕
    active: Mutex<usize>,
    pins: Mutex<HashMap<u32, Pinned>>,
    next_pin: AtomicU32,
    /// 上次保存的目录，下次保存时默认打开这里
    last_dir: Mutex<Option<PathBuf>>,
    /// 从主窗口发起的截图：主窗口先藏起来，截完再显示
    pub restore_main: AtomicBool,
}

fn err(e: impl std::fmt::Display) -> String {
    e.to_string()
}

// ---- 截屏和窗口列表 ----

#[cfg(target_os = "macos")]
mod mac {
    use super::Rect;
    use objc2::rc::Retained;
    use objc2::runtime::AnyObject;
    use objc2_foundation::{NSArray, NSDictionary, NSNumber, NSString, ns_string};
    use std::ffi::c_void;

    #[link(name = "CoreGraphics", kind = "framework")]
    unsafe extern "C" {
        fn CGPreflightScreenCaptureAccess() -> bool;
        fn CGRequestScreenCaptureAccess() -> bool;
        fn CGWindowListCopyWindowInfo(option: u32, relative_to: u32) -> *mut c_void;
    }

    pub fn has_permission() -> bool {
        unsafe { CGPreflightScreenCaptureAccess() }
    }

    /// 弹出系统的授权提示，并把应用加进“屏幕录制”列表
    pub fn request_permission() {
        unsafe { CGRequestScreenCaptureAccess() };
    }

    fn num(d: &NSDictionary<NSString, AnyObject>, key: &NSString) -> Option<f64> {
        d.objectForKey(key)?.downcast::<NSNumber>().ok().map(|n| n.doubleValue())
    }

    /// 屏幕上可见的普通窗口，从最前面到最后面，坐标是全局逻辑点。
    /// 一次取完所有信息：xcap 每读一个属性都会重新拉一遍窗口列表，窗口多时要几百毫秒
    pub fn windows() -> Vec<(Rect, u32)> {
        // kCGWindowListOptionOnScreenOnly | kCGWindowListExcludeDesktopElements
        let ptr = unsafe { CGWindowListCopyWindowInfo(1 | 16, 0) };
        // CFArray 和 NSArray 可以直接互换；Copy 开头的函数返回的已经持有一次引用
        let Some(list) = (unsafe { Retained::<NSArray<NSDictionary<NSString, AnyObject>>>::from_raw(ptr.cast()) }) else {
            return vec![];
        };
        let mut out = vec![];
        for d in list.iter() {
            // 只要第 0 层的普通窗口：菜单栏、Dock、状态栏图标、通知，还有截图窗口和贴图都在别的层
            if num(&d, ns_string!("kCGWindowLayer")) != Some(0.0) {
                continue;
            }
            let pid = num(&d, ns_string!("kCGWindowOwnerPID")).unwrap_or(0.0) as u32;
            let Some(b) = d.objectForKey(ns_string!("kCGWindowBounds")).and_then(|b| b.downcast::<NSDictionary>().ok()) else {
                continue;
            };
            // SAFETY: kCGWindowBounds 的键和值都是字符串和数字
            let b: Retained<NSDictionary<NSString, AnyObject>> = unsafe { Retained::cast_unchecked(b) };
            let r = Rect {
                x: num(&b, ns_string!("X")).unwrap_or(0.0),
                y: num(&b, ns_string!("Y")).unwrap_or(0.0),
                w: num(&b, ns_string!("Width")).unwrap_or(0.0),
                h: num(&b, ns_string!("Height")).unwrap_or(0.0),
            };
            out.push((r, pid));
        }
        out
    }
}

#[cfg(not(target_os = "macos"))]
fn list_windows() -> Vec<(Rect, u32)> {
    // Windows 上 xcap 按 Z 顺序从前到后返回，坐标是物理像素。
    // 截图窗口和贴图窗口按标题排除（主窗口要能截）
    xcap::Window::all()
        .unwrap_or_default()
        .into_iter()
        .filter(|w| !w.is_minimized().unwrap_or(true))
        .filter(|w| {
            let title = w.title().unwrap_or_default();
            !title.is_empty() && !title.starts_with("ToolTree Screenshot") && !title.starts_with("ToolTree Pin")
        })
        .filter_map(|w| {
            let r = Rect { x: w.x().ok()? as f64, y: w.y().ok()? as f64, w: w.width().ok()? as f64, h: w.height().ok()? as f64 };
            Some((r, w.pid().unwrap_or(0)))
        })
        .collect()
}

#[cfg(target_os = "macos")]
fn list_windows() -> Vec<(Rect, u32)> {
    mac::windows()
}

/// 截下所有屏幕，同时取窗口列表。
/// skip_self：主窗口刚被藏起来时，系统的窗口列表里可能还有它，把自己的窗口都排除掉
fn capture(skip_self: bool) -> Result<Vec<Screen>, String> {
    let count = xcap::Monitor::all().map_err(err)?.len();
    let windows = std::thread::spawn(list_windows);
    // 每个屏幕一个线程同时截。Windows 上 Monitor 里是句柄，不能传给别的线程，所以各线程按序号自己取
    let handles: Vec<_> = (0..count)
        .map(|i| {
            std::thread::spawn(move || -> Result<Screen, String> {
                let m = xcap::Monitor::all().map_err(err)?.into_iter().nth(i).ok_or("显示器数量变了，请重试")?;
                let image = m.capture_image().map_err(err)?;
                let scale = m.scale_factor().unwrap_or(1.0) as f64;
                let to_css = if cfg!(target_os = "macos") { 1.0 } else { 1.0 / scale };
                Ok(Screen {
                    x: m.x().map_err(err)?,
                    y: m.y().map_err(err)?,
                    w: m.width().map_err(err)?,
                    h: m.height().map_err(err)?,
                    to_css,
                    image,
                    windows: vec![],
                })
            })
        })
        .collect();
    let mut screens = handles
        .into_iter()
        .map(|h| h.join().map_err(|_| "截屏线程出错".to_string())?)
        .collect::<Result<Vec<_>, _>>()?;
    let me = std::process::id();
    let windows: Vec<Rect> = windows.join().unwrap_or_default().into_iter().filter(|(_, pid)| !(skip_self && *pid == me)).map(|(r, _)| r).collect();
    for s in &mut screens {
        let (sx, sy, sw, sh) = (s.x as f64, s.y as f64, s.w as f64, s.h as f64);
        s.windows = windows
            .iter()
            .filter(|r| r.w >= 20.0 && r.h >= 20.0)
            // 和这个屏幕有交集的，裁到屏幕范围内
            .filter_map(|r| {
                let x0 = r.x.max(sx);
                let y0 = r.y.max(sy);
                let x1 = (r.x + r.w).min(sx + sw);
                let y1 = (r.y + r.h).min(sy + sh);
                (x1 > x0 && y1 > y0).then_some(Rect {
                    x: (x0 - sx) * s.to_css,
                    y: (y0 - sy) * s.to_css,
                    w: (x1 - x0) * s.to_css,
                    h: (y1 - y0) * s.to_css,
                })
            })
            .collect();
    }
    Ok(screens)
}

// ---- 截图窗口 ----

/// 把窗口放到最上层，盖住菜单栏和 Dock，并且在全屏应用的桌面上也能显示
#[cfg(target_os = "macos")]
fn raise(w: &WebviewWindow, level: isize) {
    use objc2_app_kit::{NSWindow, NSWindowCollectionBehavior};
    let Ok(ptr) = w.ns_window() else { return };
    let ptr = ptr as usize;
    let _ = w.run_on_main_thread(move || {
        // SAFETY: Tauri 给的 NSWindow 指针，在主线程上用
        let win = unsafe { &*(ptr as *const NSWindow) };
        win.setLevel(level);
        win.setCollectionBehavior(
            NSWindowCollectionBehavior::CanJoinAllSpaces
                | NSWindowCollectionBehavior::FullScreenAuxiliary
                | NSWindowCollectionBehavior::Stationary,
        );
    });
}

#[cfg(not(target_os = "macos"))]
fn raise(_: &WebviewWindow, _: isize) {}

/// 状态栏是 25 层，盖过它但低于输入法候选框，标注文字时还能用输入法
const OVERLAY_LEVEL: isize = 26;
/// 浮动窗口层，贴图在普通窗口之上
const PIN_LEVEL: isize = 3;

fn overlay_window(app: &AppHandle, i: usize) -> Result<WebviewWindow, String> {
    let label = format!("{OVERLAY}{i}");
    if let Some(w) = app.get_webview_window(&label) {
        return Ok(w);
    }
    WebviewWindowBuilder::new(app, &label, WebviewUrl::App(format!("index.html#shot/{i}").into()))
        .title("ToolTree Screenshot")
        .decorations(false)
        .resizable(false)
        .always_on_top(true)
        .skip_taskbar(true)
        .shadow(false)
        .visible(false)
        .focused(false)
        .visible_on_all_workspaces(true)
        .background_color(tauri::window::Color(0, 0, 0, 255))
        .build()
        .map_err(err)
}

fn place(w: &WebviewWindow, s: &Screen) {
    if cfg!(target_os = "macos") {
        let _ = w.set_position(tauri::LogicalPosition::new(s.x, s.y));
        let _ = w.set_size(tauri::LogicalSize::new(s.w, s.h));
    } else {
        let _ = w.set_position(tauri::PhysicalPosition::new(s.x, s.y));
        let _ = w.set_size(tauri::PhysicalSize::new(s.w, s.h));
    }
}

/// 开始截图。截屏要几十到几百毫秒，放在后台线程里做，不卡住界面
pub fn start(app: &AppHandle) {
    let state = app.state::<ShotState>();
    if state.busy.swap(true, Ordering::SeqCst) {
        return;
    }
    let app = app.clone();
    std::thread::spawn(move || {
        if let Err(e) = run(&app) {
            end(&app);
            eprintln!("截图失败：{e}");
            let _ = app.emit_to(crate::MAIN, "toast", format!("截图失败：{e}"));
        }
    });
}

fn run(app: &AppHandle) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    if !mac::has_permission() {
        app.state::<ShotState>().restore_main.store(false, Ordering::SeqCst);
        mac::request_permission();
        crate::show_main_window(app);
        let _ = app.emit_to(crate::MAIN, "shot-permission", ());
        // 主窗口里会提示怎么授权，不再当成错误弹提示
        app.state::<ShotState>().busy.store(false, Ordering::SeqCst);
        return Ok(());
    }
    let state = app.state::<ShotState>();
    *state.previous.lock().unwrap() = Some(platform::remember_frontmost());
    let main_visible = app.get_webview_window(crate::MAIN).and_then(|w| w.is_visible().ok()).unwrap_or(false);
    let screens = capture(!main_visible)?;
    let cursor = app.cursor_position().ok();
    let active = cursor
        .and_then(|c| {
            screens.iter().position(|s| {
                // cursor_position 是物理像素；macOS 上屏幕坐标是逻辑点，要乘缩放比例
                let k = if cfg!(target_os = "macos") { s.image.width() as f64 / s.w as f64 } else { 1.0 };
                let (x0, y0) = (s.x as f64 * k, s.y as f64 * k);
                c.x >= x0 && c.x < x0 + s.w as f64 * k && c.y >= y0 && c.y < y0 + s.h as f64 * k
            })
        })
        .unwrap_or(0);
    let n = screens.len();
    let windows: Vec<_> = (0..n).map(|i| overlay_window(app, i)).collect::<Result<_, _>>()?;
    for (w, s) in windows.iter().zip(&screens) {
        place(w, s);
    }
    *state.active.lock().unwrap() = active;
    *state.screens.lock().unwrap() = screens;
    state.seq.fetch_add(1, Ordering::SeqCst);
    // 网页取到图、画好之后调用 shot_ready 才显示，避免先闪一下上次的画面
    for w in &windows {
        let _ = w.emit_to(w.label(), "shot-start", ());
    }
    Ok(())
}

/// 截图结束：隐藏所有截图窗口，释放图片，返回截图前在前台的应用
fn end(app: &AppHandle) -> Option<platform::Previous> {
    let state = app.state::<ShotState>();
    for (label, w) in app.webview_windows() {
        if label.starts_with(OVERLAY) {
            let _ = w.hide();
        }
    }
    state.screens.lock().unwrap().clear();
    state.busy.store(false, Ordering::SeqCst);
    let prev = state.previous.lock().unwrap().take();
    if state.restore_main.swap(false, Ordering::SeqCst) {
        crate::show_main_window(app);
        return None;
    }
    prev
}

/// 截图结束并切回之前的应用
fn finish(app: &AppHandle) {
    if let Some(prev) = end(app) {
        platform::activate(prev);
    }
}

fn screen_index(label: &str) -> Option<usize> {
    label.strip_prefix(OVERLAY)?.parse().ok()
}

#[tauri::command]
pub fn shot_info(w: WebviewWindow, state: State<ShotState>) -> Option<ShotInfo> {
    let i = screen_index(w.label())?;
    let screens = state.screens.lock().unwrap();
    let s = screens.get(i)?;
    Some(ShotInfo {
        id: state.seq.load(Ordering::SeqCst),
        width: s.w as f64 * s.to_css,
        height: s.h as f64 * s.to_css,
        windows: s.windows.clone(),
        active: *state.active.lock().unwrap() == i,
    })
}

/// 截到的图：前 8 个字节是宽和高（小端 u32），后面是 RGBA 像素，直接按二进制传，不经过 JSON
fn raw(image: &RgbaImage) -> Response {
    let mut out = Vec::with_capacity(8 + image.as_raw().len());
    out.extend_from_slice(&image.width().to_le_bytes());
    out.extend_from_slice(&image.height().to_le_bytes());
    out.extend_from_slice(image.as_raw());
    Response::new(out)
}

#[tauri::command]
pub async fn shot_image(w: WebviewWindow, state: State<'_, ShotState>) -> Result<Response, String> {
    let i = screen_index(w.label()).ok_or("不是截图窗口")?;
    let screens = state.screens.lock().unwrap();
    screens.get(i).map(|s| raw(&s.image)).ok_or_else(|| "截图已经结束".into())
}

/// 网页已经把截图画好了，显示窗口
#[tauri::command]
pub fn shot_ready(w: WebviewWindow, state: State<ShotState>) {
    let Some(i) = screen_index(w.label()) else { return };
    if !state.busy.load(Ordering::SeqCst) {
        return;
    }
    let _ = w.show();
    raise(&w, OVERLAY_LEVEL);
    if *state.active.lock().unwrap() == i {
        let _ = w.set_focus();
    }
}

#[tauri::command]
pub fn shot_cancel(app: AppHandle) {
    finish(&app);
}

fn header<'a>(req: &'a Request<'_>, name: &str) -> &'a str {
    req.headers().get(name).and_then(|v| v.to_str().ok()).unwrap_or("")
}

fn header_f64(req: &Request<'_>, name: &str) -> f64 {
    header(req, name).parse().unwrap_or(0.0)
}

/// 网页发回的结果：二进制 RGBA（同样是 8 个字节宽高开头），请求头里带上要做什么
fn decode(req: &Request<'_>) -> Result<RgbaImage, String> {
    let InvokeBody::Raw(bytes) = req.body() else { return Err("没有图片数据".into()) };
    if bytes.len() < 8 {
        return Err("图片数据不完整".into());
    }
    let w = u32::from_le_bytes(bytes[0..4].try_into().unwrap());
    let h = u32::from_le_bytes(bytes[4..8].try_into().unwrap());
    RgbaImage::from_raw(w, h, bytes[8..].to_vec()).ok_or_else(|| "图片数据不完整".into())
}

pub fn copy_image(img: &RgbaImage) -> Result<(), String> {
    let data = arboard::ImageData {
        width: img.width() as usize,
        height: img.height() as usize,
        bytes: std::borrow::Cow::Borrowed(img.as_raw()),
    };
    arboard::Clipboard::new().and_then(|mut c| c.set_image(data)).map_err(err)
}

fn default_name() -> String {
    format!("ToolTree_{}.png", chrono::Local::now().format("%Y%m%d_%H%M%S"))
}

/// 弹出保存对话框并保存成 PNG。返回保存的路径，取消时返回 None
fn save_image(app: &AppHandle, img: &RgbaImage) -> Result<Option<PathBuf>, String> {
    use tauri_plugin_dialog::DialogExt;
    let state = app.state::<ShotState>();
    let dir = state.last_dir.lock().unwrap().clone().or_else(|| app.path().picture_dir().ok()).or_else(|| app.path().desktop_dir().ok());
    let mut dlg = app.dialog().file().set_file_name(default_name()).add_filter("PNG", &["png"]);
    if let Some(d) = dir {
        dlg = dlg.set_directory(d);
    }
    platform::activate_self();
    let Some(path) = dlg.blocking_save_file() else { return Ok(None) };
    let path = path.into_path().map_err(err)?;
    img.save_with_format(&path, image::ImageFormat::Png).map_err(err)?;
    *state.last_dir.lock().unwrap() = path.parent().map(|p| p.to_path_buf());
    Ok(Some(path))
}

/// 截图完成：action 是 copy / save / pin
#[tauri::command]
pub async fn shot_finish(app: AppHandle, w: WebviewWindow, request: Request<'_>) -> Result<(), String> {
    let action = header(&request, "action").to_string();
    let img = decode(&request)?;
    // 贴图放在选区原来的位置：x、y 是选区在这个屏幕上的 CSS 坐标
    let (x, y) = (header_f64(&request, "x"), header_f64(&request, "y"));
    let (css_w, css_h) = (header_f64(&request, "w"), header_f64(&request, "h"));
    let origin = {
        let state = app.state::<ShotState>();
        let screens = state.screens.lock().unwrap();
        screen_index(w.label()).and_then(|i| screens.get(i)).map(|s| (s.x, s.y, s.to_css))
    };
    // 保存时要弹对话框，先不切回之前的应用，保存完再切
    let prev = end(&app);
    let restore = move || {
        if let Some(p) = prev {
            platform::activate(p);
        }
    };
    match action.as_str() {
        "copy" => {
            restore();
            copy_image(&img)
        }
        "save" => {
            let app2 = app.clone();
            let r = tauri::async_runtime::spawn_blocking(move || save_image(&app2, &img)).await.map_err(err)?;
            restore();
            r.map(|_| ())
        }
        "pin" => {
            restore();
            let (ox, oy, to_css) = origin.unwrap_or((0, 0, 1.0));
            // 换回窗口定位用的单位
            let gx = ox as f64 + x / to_css;
            let gy = oy as f64 + y / to_css;
            pin(&app, img, (gx, gy), (css_w, css_h))
        }
        _ => {
            restore();
            Ok(())
        }
    }
}

// ---- 贴图 ----

/// 把图片钉在桌面上：一个置顶的无边框小窗口。pos 是窗口位置（单位同屏幕坐标），size 是 CSS 像素大小
pub fn pin(app: &AppHandle, image: RgbaImage, pos: (f64, f64), size: (f64, f64)) -> Result<(), String> {
    let state = app.state::<ShotState>();
    let id = state.next_pin.fetch_add(1, Ordering::SeqCst);
    state.pins.lock().unwrap().insert(id, Pinned { image });
    let label = format!("{PIN}{id}");
    let w = WebviewWindowBuilder::new(app, &label, WebviewUrl::App(format!("index.html#pin/{id}").into()))
        .title("ToolTree Pin")
        .decorations(false)
        .resizable(false)
        .always_on_top(true)
        .skip_taskbar(true)
        .shadow(true)
        .visible(false)
        .visible_on_all_workspaces(true)
        .inner_size(size.0.max(20.0), size.1.max(20.0))
        .build()
        .map_err(err)?;
    if cfg!(target_os = "macos") {
        let _ = w.set_position(tauri::LogicalPosition::new(pos.0, pos.1));
    } else {
        let _ = w.set_position(tauri::PhysicalPosition::new(pos.0 as i32, pos.1 as i32));
    }
    raise(&w, PIN_LEVEL);
    Ok(())
}

fn pin_id(label: &str) -> Option<u32> {
    label.strip_prefix(PIN)?.parse().ok()
}

#[tauri::command]
pub async fn pin_image(w: WebviewWindow, state: State<'_, ShotState>) -> Result<Response, String> {
    let id = pin_id(w.label()).ok_or("不是贴图窗口")?;
    let pins = state.pins.lock().unwrap();
    pins.get(&id).map(|p| raw(&p.image)).ok_or_else(|| "贴图已经关闭".into())
}

#[tauri::command]
pub async fn pin_action(app: AppHandle, w: WebviewWindow, action: String) -> Result<(), String> {
    let id = pin_id(w.label()).ok_or("不是贴图窗口")?;
    let img = {
        let state = app.state::<ShotState>();
        let pins = state.pins.lock().unwrap();
        pins.get(&id).map(|p| p.image.clone())
    };
    let Some(img) = img else { return Ok(()) };
    match action.as_str() {
        "copy" => copy_image(&img),
        "save" => {
            let app2 = app.clone();
            tauri::async_runtime::spawn_blocking(move || save_image(&app2, &img)).await.map_err(err)?.map(|_| ())
        }
        "close" => {
            app.state::<ShotState>().pins.lock().unwrap().remove(&id);
            w.destroy().map_err(err)
        }
        _ => Ok(()),
    }
}

/// 截图窗口提前建好，第一次截图也不用等网页加载
pub fn prepare(app: &AppHandle) {
    let n = xcap::Monitor::all().map(|m| m.len()).unwrap_or(1);
    for i in 0..n {
        if let Err(e) = overlay_window(app, i) {
            eprintln!("创建截图窗口失败：{e}");
        }
    }
}
