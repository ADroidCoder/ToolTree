//! 和系统打交道的部分：剪贴板变化计数、前台应用、模拟粘贴、辅助功能权限。
//!
//! 剪贴板没有跨平台的“内容变了”通知，各系统都有一个每次变化加一的计数，轮询它几乎没有开销，
//! 计数变了才去读内容。Linux 上没有这样的计数，返回 None，由调用方比较文本内容。

/// 之前在前台的应用，粘贴时要切回去
#[derive(Clone, Copy, Debug)]
pub enum Previous {
    #[cfg(target_os = "macos")]
    Pid(i32),
    #[cfg(target_os = "windows")]
    Hwnd(isize),
    #[allow(dead_code)]
    None,
}

#[cfg(target_os = "macos")]
mod imp {
    use super::Previous;
    use objc2_app_kit::{NSApplicationActivationOptions, NSPasteboard, NSRunningApplication, NSWorkspace};

    pub fn change_count() -> Option<i64> {
        Some(NSPasteboard::generalPasteboard().changeCount() as i64)
    }

    /// 密码管理器等应用会给内容打上这些标记，表示不应该被记录
    /// （约定见 nspasteboard.org）
    pub fn is_concealed() -> bool {
        let Some(types) = NSPasteboard::generalPasteboard().types() else { return false };
        types.iter().any(|t| {
            let s = t.to_string();
            s == "org.nspasteboard.ConcealedType" || s == "org.nspasteboard.TransientType" || s == "com.agilebits.onepassword"
        })
    }

    fn frontmost() -> Option<objc2::rc::Retained<NSRunningApplication>> {
        NSWorkspace::sharedWorkspace().frontmostApplication()
    }

    pub fn frontmost_name() -> String {
        frontmost().and_then(|a| a.localizedName()).map(|s| s.to_string()).unwrap_or_default()
    }

    pub fn remember_frontmost() -> Previous {
        match frontmost() {
            Some(a) if a.processIdentifier() != std::process::id() as i32 => Previous::Pid(a.processIdentifier()),
            _ => Previous::None,
        }
    }

    /// 把自己切到前台（弹出保存对话框前用，否则对话框会被别的应用挡住）
    pub fn activate_self() {
        NSRunningApplication::currentApplication().activateWithOptions(NSApplicationActivationOptions::ActivateAllWindows);
    }

    pub fn activate(prev: Previous) {
        if let Previous::Pid(pid) = prev
            && let Some(a) = NSRunningApplication::runningApplicationWithProcessIdentifier(pid)
        {
            a.activateWithOptions(NSApplicationActivationOptions::empty());
        }
    }

    #[link(name = "ApplicationServices", kind = "framework")]
    unsafe extern "C" {
        fn AXIsProcessTrusted() -> bool;
    }

    /// 模拟按键需要“辅助功能”权限
    pub fn accessibility_trusted() -> bool {
        unsafe { AXIsProcessTrusted() }
    }

    pub fn open_accessibility_settings() {
        let _ = std::process::Command::new("open")
            .arg("x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility")
            .spawn();
    }

    pub fn open_screen_recording_settings() {
        let _ = std::process::Command::new("open")
            .arg("x-apple.systempreferences:com.apple.preference.security?Privacy_ScreenCapture")
            .spawn();
    }
}

#[cfg(target_os = "windows")]
mod imp {
    use super::Previous;
    use windows_sys::Win32::System::DataExchange::GetClipboardSequenceNumber;
    use windows_sys::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, GetWindowThreadProcessId, SetForegroundWindow};

    pub fn change_count() -> Option<i64> {
        Some(unsafe { GetClipboardSequenceNumber() } as i64)
    }

    pub fn is_concealed() -> bool {
        false
    }

    pub fn frontmost_name() -> String {
        String::new()
    }

    pub fn remember_frontmost() -> Previous {
        unsafe {
            let hwnd = GetForegroundWindow();
            let mut pid = 0u32;
            GetWindowThreadProcessId(hwnd, &mut pid);
            if hwnd.is_null() || pid == std::process::id() { Previous::None } else { Previous::Hwnd(hwnd as isize) }
        }
    }

    pub fn activate_self() {}

    pub fn activate(prev: Previous) {
        if let Previous::Hwnd(h) = prev {
            unsafe { SetForegroundWindow(h as _) };
        }
    }

    pub fn accessibility_trusted() -> bool {
        true
    }

    pub fn open_accessibility_settings() {}

    pub fn open_screen_recording_settings() {}
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
mod imp {
    use super::Previous;

    pub fn change_count() -> Option<i64> {
        None
    }

    pub fn is_concealed() -> bool {
        false
    }

    pub fn frontmost_name() -> String {
        String::new()
    }

    pub fn remember_frontmost() -> Previous {
        Previous::None
    }

    pub fn activate_self() {}

    pub fn activate(_: Previous) {}

    pub fn accessibility_trusted() -> bool {
        true
    }

    pub fn open_accessibility_settings() {}

    pub fn open_screen_recording_settings() {}
}

pub use imp::*;

/// 模拟按下粘贴键（macOS 上 ⌘V，其他系统 Ctrl+V）
pub fn send_paste() -> Result<(), String> {
    use enigo::{Direction, Enigo, Key, Keyboard, Settings};
    let mut e = Enigo::new(&Settings::default()).map_err(|e| e.to_string())?;
    #[cfg(target_os = "macos")]
    {
        // 用 V 键的键码而不是字符：按字符要查当前键盘布局，那个系统接口只能在主线程调用，
        // 而且换成别的布局（比如 Dvorak）时字符 v 不在同一个键上
        e.key(Key::Meta, Direction::Press).map_err(|e| e.to_string())?;
        let r = e.raw(9, Direction::Click);
        e.key(Key::Meta, Direction::Release).map_err(|e| e.to_string())?;
        r.map_err(|e| e.to_string())
    }
    #[cfg(not(target_os = "macos"))]
    {
        e.key(Key::Control, Direction::Press).map_err(|e| e.to_string())?;
        let r = e.key(Key::Unicode('v'), Direction::Click);
        e.key(Key::Control, Direction::Release).map_err(|e| e.to_string())?;
        r.map_err(|e| e.to_string())
    }
}
