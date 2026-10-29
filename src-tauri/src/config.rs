//! 保存在配置目录 config.json 里的设置：全局快捷键和剪贴板历史。
//! 界面外观（主题、语言）由前端存在 localStorage 里，不在这里。

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Config {
    /// 打开工具搜索的全局快捷键，空表示不用
    pub launcher_shortcut: String,
    /// 打开剪贴板历史面板的全局快捷键
    pub clip_shortcut: String,
    /// 截图的全局快捷键
    pub shot_shortcut: String,
    pub clip_enabled: bool,
    /// 最多保留多少条（置顶的不算）
    pub clip_max: i64,
    /// 保留多少天，0 表示不限
    pub clip_days: i64,
    /// 在面板里选中后直接粘贴到之前的应用
    pub auto_paste: bool,
    /// 不记录这些应用里复制的内容（按应用名，不区分大小写）
    pub ignore_apps: Vec<String>,
    pub translate: TranslateConfig,
}

/// 翻译用哪家，以及各家的密钥
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct TranslateConfig {
    /// free（Google，不通时用 MyMemory）、tencent、aliyun、deepl
    pub provider: String,
    pub tencent_id: String,
    pub tencent_key: String,
    pub aliyun_id: String,
    pub aliyun_secret: String,
    pub deepl_key: String,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            // Windows 上 Alt+Space 是窗口的系统菜单，换成 Ctrl+Alt+Space
            launcher_shortcut: if cfg!(target_os = "macos") { "Alt+Space" } else { "Control+Alt+Space" }.into(),
            clip_shortcut: "CommandOrControl+Shift+V".into(),
            // macOS 的 ⌘⇧3/4/5 是系统截图，⌘⇧1 没被占用；其他系统和 Snipaste 一样用 F1
            shot_shortcut: if cfg!(target_os = "macos") { "Command+Shift+1" } else { "F1" }.into(),
            clip_enabled: true,
            clip_max: 1000,
            clip_days: 30,
            auto_paste: true,
            // 密码管理器：它们大多会标记“隐藏内容”，本来就不会记录，这里再保险一下
            ignore_apps: vec!["1Password".into(), "Bitwarden".into(), "KeePassXC".into(), "Keychain Access".into(), "钥匙串访问".into()],
            translate: TranslateConfig::default(),
        }
    }
}

pub fn path(dir: &Path) -> PathBuf {
    dir.join("config.json")
}

pub fn load(dir: &Path) -> Config {
    fs::read_to_string(path(dir))
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

pub fn save(dir: &Path, c: &Config) -> Result<(), String> {
    fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    let s = serde_json::to_string_pretty(c).map_err(|e| e.to_string())?;
    // 先写临时文件再改名，写到一半退出也不会把配置弄坏
    let tmp = dir.join("config.json.tmp");
    fs::write(&tmp, s).map_err(|e| e.to_string())?;
    fs::rename(tmp, path(dir)).map_err(|e| e.to_string())
}
