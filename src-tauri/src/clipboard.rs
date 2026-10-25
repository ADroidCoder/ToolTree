//! 剪贴板监听：后台线程轮询系统的变化计数，变了就读内容存进历史，然后通知界面刷新。

use crate::config::Config;
use crate::platform;
use arboard::{Clipboard, ImageData};
use std::borrow::Cow;
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::{Arc, RwLock};
use std::time::Duration;
use tauri::{AppHandle, Emitter};
use tooltree_core::clips::{Clip, ClipStore, Kind};

/// 历史有变化（新增、删除、置顶）时发给所有窗口的事件
pub const CHANGED: &str = "clips-changed";

const POLL: Duration = Duration::from_millis(400);
/// 每新增这么多条清理一次过期的记录
const PRUNE_EVERY: u32 = 50;

pub struct Watcher {
    pub store: Arc<ClipStore>,
    pub config: Arc<RwLock<Config>>,
    /// 最近一次看到的变化计数。自己写剪贴板后也更新它，免得把写回去的内容又记一遍
    last_seq: AtomicI64,
}

impl Watcher {
    pub fn new(store: Arc<ClipStore>, config: Arc<RwLock<Config>>) -> Arc<Self> {
        Arc::new(Watcher { store, config, last_seq: AtomicI64::new(platform::change_count().unwrap_or(-1)) })
    }

    pub fn start(self: &Arc<Self>, app: AppHandle) {
        let me = self.clone();
        std::thread::Builder::new()
            .name("clipboard".into())
            .spawn(move || me.run(app))
            .expect("启动剪贴板监听失败");
    }

    fn run(&self, app: AppHandle) {
        let mut cb: Option<Clipboard> = None;
        // 没有变化计数的系统（Linux）比较文本内容
        let mut last_text = String::new();
        let mut added = 0u32;
        loop {
            std::thread::sleep(POLL);
            let seq = platform::change_count();
            if let Some(s) = seq
                && self.last_seq.swap(s, Ordering::SeqCst) == s {
                    continue;
                }
            let (enabled, ignore) = {
                let c = self.config.read().unwrap();
                (c.clip_enabled, c.ignore_apps.clone())
            };
            if !enabled || platform::is_concealed() {
                continue;
            }
            let source = platform::frontmost_name();
            if !source.is_empty() && ignore.iter().any(|a| a.eq_ignore_ascii_case(&source)) {
                continue;
            }
            if cb.is_none() {
                cb = Clipboard::new().ok();
            }
            let Some(c) = cb.as_mut() else { continue };
            if seq.is_none() {
                let text = c.get_text().unwrap_or_default();
                if text == last_text {
                    continue;
                }
                last_text = text;
            }
            match self.read(c, &source) {
                Ok(Some(_)) => {
                    added += 1;
                    if added.is_multiple_of(PRUNE_EVERY) {
                        self.prune();
                    }
                    let _ = app.emit(CHANGED, ());
                }
                Ok(None) => {}
                Err(e) => eprintln!("记录剪贴板失败：{e}"),
            }
        }
    }

    /// 读出当前剪贴板并存起来。复制文件时剪贴板里同时有文件名文本，所以先看文件；
    /// 从 Office 复制文字时同时有一张渲染好的图片，所以文字优先于图片
    fn read(&self, cb: &mut Clipboard, source: &str) -> Result<Option<Clip>, String> {
        if let Ok(files) = cb.get().file_list()
            && !files.is_empty()
        {
            let paths: Vec<String> = files.iter().map(|p| p.to_string_lossy().into_owned()).collect();
            return self.store.add_files(&paths, source);
        }
        if let Ok(text) = cb.get_text()
            && !text.trim().is_empty()
        {
            return self.store.add_text(&text, source);
        }
        if let Ok(img) = cb.get_image() {
            return self.store.add_image(img.width as u32, img.height as u32, &img.bytes, source);
        }
        Ok(None)
    }

    pub fn prune(&self) {
        let (max, days) = {
            let c = self.config.read().unwrap();
            (c.clip_max, c.clip_days)
        };
        if let Err(e) = self.store.prune(max, days) {
            eprintln!("清理剪贴板历史失败：{e}");
        }
    }

    /// 把一条历史写回系统剪贴板，并把它移到最前面
    pub fn write(&self, id: i64) -> Result<(), String> {
        let clip = self.store.get(id)?.ok_or("这条记录已经被删除了")?;
        let mut cb = Clipboard::new().map_err(|e| e.to_string())?;
        match clip.kind {
            Kind::Text => cb.set_text(clip.text.clone()),
            Kind::Files => cb.set().file_list(&clip.text.lines().collect::<Vec<_>>()),
            Kind::Image => {
                let (w, h, rgba) = self.store.load_rgba(&clip)?;
                cb.set_image(ImageData { width: w as usize, height: h as usize, bytes: Cow::Owned(rgba) })
            }
        }
        .map_err(|e| e.to_string())?;
        self.skip_current();
        self.store.touch(id)
    }

    /// 写入一段文本（工具里的“复制”按钮）。这种内容照常记进历史
    pub fn write_text(&self, text: &str) -> Result<(), String> {
        Clipboard::new().and_then(|mut c| c.set_text(text)).map_err(|e| e.to_string())
    }

    /// 当前剪贴板的内容是自己写的，不再记录
    fn skip_current(&self) {
        if let Some(s) = platform::change_count() {
            self.last_seq.store(s, Ordering::SeqCst);
        }
    }
}
