mod clipboard;
mod config;
mod launcher;
mod platform;
mod shot;

use clipboard::Watcher;
use config::Config;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, RwLock};
use std::time::Duration;
use tauri::ipc::Response;
use tauri::menu::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::{AppHandle, Emitter, Manager, PhysicalPosition, State, WindowEvent};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};
use tooltree_core::clips::{Clip, ClipStore, Query};
use tooltree_core::hash::{self, Hashes};

type CmdResult<T> = Result<T, String>;

pub(crate) const MAIN: &str = "main";
const PANEL: &str = "panel";
const TRAY: &str = "tray";
/// 开机自启时带上这个参数，只在托盘里待命，不弹出主窗口
const HIDDEN_ARG: &str = "--hidden";

struct AppState {
    dir: PathBuf,
    config: Arc<RwLock<Config>>,
    watcher: Arc<Watcher>,
    /// 当前注册的全局快捷键和对应的动作
    shortcuts: Mutex<Vec<(Shortcut, Action)>>,
    /// 打开剪贴板面板之前在前台的应用
    previous: Mutex<platform::Previous>,
}

#[derive(Clone, Copy)]
enum Action {
    Launcher,
    Clipboard,
    Screenshot,
}

/// 托盘菜单的语言，跟着界面设置
static EN: AtomicBool = AtomicBool::new(false);

fn tr<'a>(zh: &'a str, en: &'a str) -> &'a str {
    if EN.load(Ordering::Relaxed) { en } else { zh }
}

fn err(e: impl std::fmt::Display) -> String {
    e.to_string()
}

// ---- 设置 ----

#[tauri::command]
fn config_get(state: State<AppState>) -> Config {
    state.config.read().unwrap().clone()
}

#[tauri::command]
fn config_set(app: AppHandle, state: State<AppState>, config: Config) -> CmdResult<()> {
    config::save(&state.dir, &config)?;
    *state.config.write().unwrap() = config;
    state.watcher.prune();
    let _ = app.emit(clipboard::CHANGED, ());
    refresh_tray(&app);
    register_shortcuts(&app)
}

#[tauri::command]
fn set_lang(app: AppHandle, en: bool) {
    EN.store(en, Ordering::Relaxed);
    refresh_tray(&app);
}

// ---- 剪贴板历史 ----

#[tauri::command]
fn clip_list(state: State<AppState>, query: Query) -> CmdResult<Vec<Clip>> {
    state.watcher.store.list(&query)
}

#[tauri::command]
fn clip_get(state: State<AppState>, id: i64) -> CmdResult<Option<Clip>> {
    state.watcher.store.get(id)
}

#[tauri::command]
fn clip_pin(app: AppHandle, state: State<AppState>, id: i64, pinned: bool) -> CmdResult<()> {
    state.watcher.store.set_pinned(id, pinned)?;
    let _ = app.emit(clipboard::CHANGED, ());
    Ok(())
}

#[tauri::command]
fn clip_remove(app: AppHandle, state: State<AppState>, id: i64) -> CmdResult<()> {
    state.watcher.store.remove(id)?;
    let _ = app.emit(clipboard::CHANGED, ());
    Ok(())
}

#[tauri::command]
fn clip_clear(app: AppHandle, state: State<AppState>) -> CmdResult<usize> {
    let n = state.watcher.store.clear()?;
    let _ = app.emit(clipboard::CHANGED, ());
    Ok(n)
}

#[tauri::command]
fn clip_count(state: State<AppState>) -> CmdResult<i64> {
    state.watcher.store.count()
}

/// 图片原图或缩略图的 PNG 数据，直接按二进制返回，不经过 JSON
#[tauri::command]
async fn clip_image(state: State<'_, AppState>, id: i64, thumb: bool) -> CmdResult<Response> {
    let store = &state.watcher.store;
    let clip = store.get(id)?.ok_or("这条记录已经被删除了")?;
    let path = if thumb && store.thumb_path(&clip).exists() { store.thumb_path(&clip) } else { store.image_path(&clip) };
    let bytes = tauri::async_runtime::spawn_blocking(move || std::fs::read(path)).await.map_err(err)?.map_err(err)?;
    Ok(Response::new(bytes))
}

#[tauri::command]
fn clip_copy(app: AppHandle, state: State<AppState>, id: i64) -> CmdResult<()> {
    state.watcher.write(id)?;
    let _ = app.emit(clipboard::CHANGED, ());
    Ok(())
}

/// 写回剪贴板，关掉面板，切回之前的应用并模拟粘贴。
/// 没有辅助功能权限（macOS）或者关了自动粘贴时只复制，返回 false
#[tauri::command]
fn clip_paste(app: AppHandle, state: State<AppState>, id: i64) -> CmdResult<bool> {
    state.watcher.write(id)?;
    let _ = app.emit(clipboard::CHANGED, ());
    let prev = *state.previous.lock().unwrap();
    hide_panel(&app);
    platform::activate(prev);
    let auto = state.config.read().unwrap().auto_paste;
    if !auto || matches!(prev, platform::Previous::None) {
        return Ok(false);
    }
    if !platform::accessibility_trusted() {
        return Ok(false);
    }
    // 等之前的应用回到前台再按粘贴键
    std::thread::spawn(|| {
        std::thread::sleep(Duration::from_millis(150));
        if let Err(e) = platform::send_paste() {
            eprintln!("模拟粘贴失败：{e}");
        }
    });
    Ok(true)
}

#[tauri::command]
fn copy_text(state: State<AppState>, text: String) -> CmdResult<()> {
    state.watcher.write_text(&text)
}

#[tauri::command]
fn accessibility_trusted() -> bool {
    platform::accessibility_trusted()
}

#[tauri::command]
fn open_accessibility_settings() {
    platform::open_accessibility_settings()
}

#[tauri::command]
fn open_screen_recording_settings() {
    platform::open_screen_recording_settings()
}

// ---- 工具 ----

#[tauri::command]
fn hash_text(text: String) -> Hashes {
    hash::hash_bytes(text.as_bytes())
}

#[tauri::command]
async fn hash_file(path: String) -> CmdResult<Hashes> {
    tauri::async_runtime::spawn_blocking(move || hash::hash_file(path.as_ref()).map_err(err))
        .await
        .map_err(err)?
}

// ---- 窗口 ----

#[derive(Clone, serde::Serialize)]
struct OpenTool {
    tool: Option<String>,
    input: Option<String>,
}

pub(crate) fn show_main_window(app: &AppHandle) {
    if let Some(w) = app.get_webview_window(MAIN) {
        let _ = w.unminimize();
        let _ = w.show();
        let _ = w.set_focus();
    }
}

#[tauri::command]
fn show_main(app: AppHandle, tool: Option<String>, input: Option<String>) {
    hide_panel(&app);
    launcher::hide(&app, false);
    show_main_window(&app);
    if tool.is_some() {
        let _ = app.emit_to(MAIN, "open-tool", OpenTool { tool, input });
    }
}

fn hide_panel(app: &AppHandle) {
    if let Some(w) = app.get_webview_window(PANEL) {
        let _ = w.hide();
    }
}

#[tauri::command]
fn panel_hide(app: AppHandle) {
    hide_panel(&app);
}

/// 鼠标所在的屏幕
pub(crate) fn cursor_monitor(app: &AppHandle) -> Option<tauri::Monitor> {
    let cursor = app.cursor_position().ok()?;
    app.available_monitors().unwrap_or_default().into_iter().find(|m| {
        let (p, s) = (m.position(), m.size());
        cursor.x >= p.x as f64 && cursor.x < (p.x + s.width as i32) as f64 && cursor.y >= p.y as f64 && cursor.y < (p.y + s.height as i32) as f64
    })
}

/// 在鼠标所在的屏幕上、靠近鼠标的位置打开剪贴板面板；已经打开就关掉
fn toggle_panel(app: &AppHandle) {
    let Some(w) = app.get_webview_window(PANEL) else { return };
    if w.is_visible().unwrap_or(false) {
        let _ = w.hide();
        return;
    }
    let state = app.state::<AppState>();
    *state.previous.lock().unwrap() = platform::remember_frontmost();
    if let (Ok(cursor), Ok(size)) = (app.cursor_position(), w.outer_size()) {
        if let Some(m) = cursor_monitor(app) {
            let area = m.work_area();
            let (min_x, min_y) = (area.position.x, area.position.y);
            let max_x = min_x + area.size.width as i32 - size.width as i32;
            let max_y = min_y + area.size.height as i32 - size.height as i32;
            let x = (cursor.x as i32 - size.width as i32 / 2).clamp(min_x, max_x.max(min_x));
            let y = (cursor.y as i32 + 12).clamp(min_y, max_y.max(min_y));
            let _ = w.set_position(PhysicalPosition::new(x, y));
        } else {
            let _ = w.center();
        }
    }
    let _ = w.show();
    let _ = w.set_focus();
    let _ = w.emit("panel-show", ());
}

// ---- 全局快捷键 ----

fn register_shortcuts(app: &AppHandle) -> CmdResult<()> {
    let state = app.state::<AppState>();
    let gs = app.global_shortcut();
    let _ = gs.unregister_all();
    let wanted = {
        let c = state.config.read().unwrap();
        [
            (c.launcher_shortcut.clone(), Action::Launcher),
            (c.clip_shortcut.clone(), Action::Clipboard),
            (c.shot_shortcut.clone(), Action::Screenshot),
        ]
    };
    let mut registered = vec![];
    let mut errors = vec![];
    for (s, action) in wanted {
        if s.trim().is_empty() {
            continue;
        }
        // 解析失败多半是写错了，注册失败多半是被别的应用占用了
        match s.parse::<Shortcut>().map_err(err).and_then(|sc| gs.register(sc).map(|_| sc).map_err(err)) {
            Ok(sc) => registered.push((sc, action)),
            Err(e) => errors.push(format!("{s}: {e}")),
        }
    }
    *state.shortcuts.lock().unwrap() = registered;
    if errors.is_empty() { Ok(()) } else { Err(errors.join("\n")) }
}

/// 设置里录制快捷键时先把全局快捷键都停掉，不然按下已占用的组合键会被系统拦走，网页收不到
#[tauri::command]
fn shortcuts_pause(app: AppHandle, paused: bool) {
    if paused {
        let _ = app.global_shortcut().unregister_all();
        app.state::<AppState>().shortcuts.lock().unwrap().clear();
    } else {
        let _ = register_shortcuts(&app);
    }
}

fn on_shortcut(app: &AppHandle, sc: &Shortcut) {
    let action = app.state::<AppState>().shortcuts.lock().unwrap().iter().find(|(s, _)| s == sc).map(|(_, a)| *a);
    match action {
        Some(Action::Launcher) => launcher::toggle(app),
        Some(Action::Clipboard) => toggle_panel(app),
        Some(Action::Screenshot) => shot::start(app),
        None => {}
    }
}

/// 主窗口里点“截图”：先把主窗口藏起来，等它消失了再截
#[tauri::command]
fn shot_start(app: AppHandle, hide_main: bool) {
    if hide_main && let Some(w) = app.get_webview_window(MAIN) {
        let _ = w.hide();
        app.state::<shot::ShotState>().restore_main.store(true, Ordering::SeqCst);
        std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(250));
            shot::start(&app);
        });
    } else {
        shot::start(&app);
    }
}

// ---- 托盘 ----

fn tray_menu(app: &AppHandle) -> tauri::Result<Menu<tauri::Wry>> {
    let enabled = app.state::<AppState>().config.read().unwrap().clip_enabled;
    Menu::with_items(
        app,
        &[
            &MenuItem::with_id(app, "show", tr("打开 ToolTree", "Open ToolTree"), true, None::<&str>)?,
            &MenuItem::with_id(app, "launcher", tr("搜索…", "Search…"), true, None::<&str>)?,
            &MenuItem::with_id(app, "clips", tr("剪贴板历史", "Clipboard History"), true, None::<&str>)?,
            &MenuItem::with_id(app, "shot", tr("截图", "Screenshot"), true, None::<&str>)?,
            &PredefinedMenuItem::separator(app)?,
            &CheckMenuItem::with_id(app, "record", tr("记录剪贴板", "Record Clipboard"), true, enabled, None::<&str>)?,
            &PredefinedMenuItem::separator(app)?,
            &MenuItem::with_id(app, "quit", tr("退出", "Quit"), true, None::<&str>)?,
        ],
    )
}

fn refresh_tray(app: &AppHandle) {
    if let (Some(tray), Ok(menu)) = (app.tray_by_id(TRAY), tray_menu(app)) {
        let _ = tray.set_menu(Some(menu));
    }
}

fn setup_tray(app: &AppHandle) -> tauri::Result<()> {
    let mut b = TrayIconBuilder::with_id(TRAY)
        .tooltip("ToolTree")
        .menu(&tray_menu(app)?)
        .on_menu_event(|app, e| match e.id.as_ref() {
            "show" => show_main_window(app),
            "launcher" => launcher::toggle(app),
            "clips" => toggle_panel(app),
            "shot" => shot::start(app),
            "record" => {
                let state = app.state::<AppState>();
                let c = {
                    let mut c = state.config.write().unwrap();
                    c.clip_enabled = !c.clip_enabled;
                    c.clone()
                };
                let _ = config::save(&state.dir, &c);
                let _ = app.emit("config-changed", ());
                refresh_tray(app);
            }
            "quit" => app.exit(0),
            _ => {}
        });
    #[cfg(target_os = "macos")]
    {
        // 菜单栏用单色模板图标，系统会按深浅色自动反色
        b = b.icon(tauri::image::Image::from_bytes(include_bytes!("../icons/tray-template.png"))?).icon_as_template(true);
    }
    #[cfg(not(target_os = "macos"))]
    {
        if let Some(icon) = app.default_window_icon() {
            b = b.icon(icon.clone());
        }
    }
    b.build(app)?;
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app = tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_autostart::Builder::new().arg(HIDDEN_ARG).build())
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, sc, e| {
                    if e.state == ShortcutState::Pressed {
                        on_shortcut(app, sc);
                    }
                })
                .build(),
        )
        .setup(|app| {
            let dir = app.path().app_config_dir()?;
            let data = app.path().app_data_dir()?;
            let config = Arc::new(RwLock::new(config::load(&dir)));
            let store = Arc::new(ClipStore::open(&data.join("clipboard"))?);
            let watcher = Watcher::new(store, config.clone());
            app.manage(AppState {
                dir,
                config,
                watcher: watcher.clone(),
                shortcuts: Mutex::new(vec![]),
                previous: Mutex::new(platform::Previous::None),
            });
            watcher.prune();
            watcher.start(app.handle().clone());
            app.manage(shot::ShotState::default());
            app.manage(launcher::LauncherState::default());
            setup_tray(app.handle())?;
            shot::prepare(app.handle());
            if let Err(e) = register_shortcuts(app.handle()) {
                eprintln!("注册全局快捷键失败：{e}");
            }
            if !std::env::args().any(|a| a == HIDDEN_ARG) {
                show_main_window(app.handle());
            }
            Ok(())
        })
        .on_window_event(|w, e| match e {
            // 关闭主窗口只是隐藏，程序留在托盘里，剪贴板照常记录
            WindowEvent::CloseRequested { api, .. } if w.label() == MAIN => {
                api.prevent_close();
                let _ = w.hide();
            }
            // 面板失去焦点（点了别的地方）就收起来
            WindowEvent::Focused(false) if w.label() == PANEL => {
                let _ = w.hide();
            }
            WindowEvent::Focused(false) if w.label() == launcher::LAUNCHER => launcher::on_blur(w.app_handle()),
            _ => {}
        })
        .invoke_handler(tauri::generate_handler![
            config_get,
            config_set,
            set_lang,
            shortcuts_pause,
            clip_list,
            clip_get,
            clip_pin,
            clip_remove,
            clip_clear,
            clip_count,
            clip_image,
            clip_copy,
            clip_paste,
            copy_text,
            accessibility_trusted,
            open_accessibility_settings,
            open_screen_recording_settings,
            hash_text,
            hash_file,
            shot_start,
            shot::shot_info,
            shot::shot_image,
            shot::shot_ready,
            shot::shot_cancel,
            shot::shot_finish,
            shot::pin_image,
            shot::pin_action,
            show_main,
            panel_hide,
            launcher::launcher_hide,
            launcher::launcher_apps,
            launcher::launcher_icon,
            launcher::launcher_files,
            launcher::launcher_open,
        ])
        .build(tauri::generate_context!())
        .expect("启动失败");

    app.run(|app, e| {
        // macOS 上点 Dock 图标时重新打开主窗口
        #[cfg(target_os = "macos")]
        if let tauri::RunEvent::Reopen { .. } = e {
            show_main_window(app);
        }
        let _ = (app, e);
    });
}
