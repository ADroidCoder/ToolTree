//! 剪贴板历史。记录存在 SQLite 里，图片另外存成 PNG 文件（原图和缩略图），数据库只记文件名。
//!
//! 同样的内容再复制一次不会多一条，而是把原来那条移到最前面，所以用内容的哈希做唯一键。

use crate::now_ms;
use rusqlite::{Connection, OptionalExtension, Row, params};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

/// 超过这个长度的文本不记录：多半是误复制的大文件内容，存下来只会拖慢搜索
pub const MAX_TEXT: usize = 2 << 20;
/// 列表里每条只带开头这么多字符，完整内容用 get 取
const PREVIEW_CHARS: i64 = 1000;
/// 缩略图最长边
const THUMB_SIZE: u32 = 360;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Kind {
    Text,
    Image,
    Files,
}

impl Kind {
    fn as_str(self) -> &'static str {
        match self {
            Kind::Text => "text",
            Kind::Image => "image",
            Kind::Files => "files",
        }
    }

    fn parse(s: &str) -> Kind {
        match s {
            "image" => Kind::Image,
            "files" => Kind::Files,
            _ => Kind::Text,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Clip {
    pub id: i64,
    pub kind: Kind,
    /// 文本内容；文件是每行一个路径；图片为空
    pub text: String,
    /// 文本的字符数、文件个数、图片的字节数
    pub size: i64,
    /// 图片宽高
    pub width: i64,
    pub height: i64,
    /// 最近一次复制的时间（毫秒）
    pub time: i64,
    pub pinned: bool,
    /// 从哪个应用复制的，取不到时为空
    pub source: String,
    #[serde(skip)]
    pub hash: String,
}

#[derive(Debug, Default, Deserialize)]
pub struct Query {
    #[serde(default)]
    pub search: String,
    pub kind: Option<Kind>,
    #[serde(default)]
    pub pinned: bool,
    #[serde(default)]
    pub offset: i64,
    #[serde(default = "default_limit")]
    pub limit: i64,
}

fn default_limit() -> i64 {
    100
}

pub struct ClipStore {
    db: Mutex<Connection>,
    images: PathBuf,
}

const COLUMNS: &str = "id, kind, substr(text, 1, ?), size, width, height, time, pinned, source, hash";
const COLUMNS_FULL: &str = "id, kind, text, size, width, height, time, pinned, source, hash";

fn from_row(r: &Row) -> rusqlite::Result<Clip> {
    Ok(Clip {
        id: r.get(0)?,
        kind: Kind::parse(&r.get::<_, String>(1)?),
        text: r.get(2)?,
        size: r.get(3)?,
        width: r.get(4)?,
        height: r.get(5)?,
        time: r.get(6)?,
        pinned: r.get(7)?,
        source: r.get(8)?,
        hash: r.get(9)?,
    })
}

fn err(e: impl std::fmt::Display) -> String {
    e.to_string()
}

fn sha256(parts: &[&[u8]]) -> String {
    let mut h = Sha256::new();
    for p in parts {
        h.update(p);
    }
    h.finalize().iter().map(|b| format!("{b:02x}")).collect()
}

impl ClipStore {
    /// 在 dir 下打开（没有就创建）clips.db 和 images 目录
    pub fn open(dir: &Path) -> Result<Self, String> {
        let images = dir.join("images");
        fs::create_dir_all(&images).map_err(err)?;
        let db = Connection::open(dir.join("clips.db")).map_err(err)?;
        db.execute_batch(
            "PRAGMA journal_mode = WAL;
             CREATE TABLE IF NOT EXISTS clips (
                 id INTEGER PRIMARY KEY AUTOINCREMENT,
                 kind TEXT NOT NULL,
                 text TEXT NOT NULL DEFAULT '',
                 size INTEGER NOT NULL DEFAULT 0,
                 width INTEGER NOT NULL DEFAULT 0,
                 height INTEGER NOT NULL DEFAULT 0,
                 time INTEGER NOT NULL,
                 pinned INTEGER NOT NULL DEFAULT 0,
                 source TEXT NOT NULL DEFAULT '',
                 hash TEXT NOT NULL UNIQUE
             );
             CREATE INDEX IF NOT EXISTS clips_time ON clips(time DESC);",
        )
        .map_err(err)?;
        Ok(ClipStore { db: Mutex::new(db), images })
    }

    pub fn image_path(&self, clip: &Clip) -> PathBuf {
        self.images.join(format!("{}.png", clip.hash))
    }

    pub fn thumb_path(&self, clip: &Clip) -> PathBuf {
        self.images.join(format!("{}.thumb.png", clip.hash))
    }

    /// 读出图片的 RGBA 像素，写回剪贴板时用
    pub fn load_rgba(&self, clip: &Clip) -> Result<(u32, u32, Vec<u8>), String> {
        let img = image::open(self.image_path(clip)).map_err(err)?.to_rgba8();
        Ok((img.width(), img.height(), img.into_raw()))
    }

    /// 新增一条；已经有同样内容时把它移到最前面。返回这条记录
    fn upsert(&self, kind: Kind, text: &str, size: i64, wh: (i64, i64), hash: &str, source: &str) -> Result<Clip, String> {
        let db = self.db.lock().unwrap();
        db.execute(
            "INSERT INTO clips (kind, text, size, width, height, time, source, hash) VALUES (?, ?, ?, ?, ?, ?, ?, ?)
             ON CONFLICT(hash) DO UPDATE SET time = excluded.time, source = excluded.source",
            params![kind.as_str(), text, size, wh.0, wh.1, now_ms(), source, hash],
        )
        .map_err(err)?;
        db.query_row(&format!("SELECT {COLUMNS} FROM clips WHERE hash = ?"), params![PREVIEW_CHARS, hash], from_row)
            .map_err(err)
    }

    /// 记录一段文本。空白内容和太长的内容不记，返回 None
    pub fn add_text(&self, text: &str, source: &str) -> Result<Option<Clip>, String> {
        if text.trim().is_empty() || text.len() > MAX_TEXT {
            return Ok(None);
        }
        let hash = sha256(&[b"text:", text.as_bytes()]);
        self.upsert(Kind::Text, text, text.chars().count() as i64, (0, 0), &hash, source).map(Some)
    }

    pub fn add_files(&self, paths: &[String], source: &str) -> Result<Option<Clip>, String> {
        if paths.is_empty() {
            return Ok(None);
        }
        let text = paths.join("\n");
        let hash = sha256(&[b"files:", text.as_bytes()]);
        self.upsert(Kind::Files, &text, paths.len() as i64, (0, 0), &hash, source).map(Some)
    }

    /// 记录一张图片（RGBA 像素）。原图和缩略图存成 PNG
    pub fn add_image(&self, width: u32, height: u32, rgba: &[u8], source: &str) -> Result<Option<Clip>, String> {
        if width == 0 || height == 0 || rgba.len() != (width * height * 4) as usize {
            return Ok(None);
        }
        let hash = sha256(&[b"image:", &width.to_le_bytes(), &height.to_le_bytes(), rgba]);
        let file = self.images.join(format!("{hash}.png"));
        if !file.exists() {
            let img = image::RgbaImage::from_raw(width, height, rgba.to_vec()).ok_or("图片数据不完整")?;
            img.save(&file).map_err(err)?;
            let thumb = if width.max(height) > THUMB_SIZE {
                image::DynamicImage::ImageRgba8(img).thumbnail(THUMB_SIZE, THUMB_SIZE).to_rgba8()
            } else {
                img
            };
            thumb.save(self.images.join(format!("{hash}.thumb.png"))).map_err(err)?;
        }
        let size = fs::metadata(&file).map(|m| m.len() as i64).unwrap_or(0);
        self.upsert(Kind::Image, "", size, (width as i64, height as i64), &hash, source).map(Some)
    }

    pub fn list(&self, q: &Query) -> Result<Vec<Clip>, String> {
        let mut sql = format!("SELECT {COLUMNS} FROM clips WHERE 1 = 1");
        let mut args: Vec<Box<dyn rusqlite::ToSql>> = vec![Box::new(PREVIEW_CHARS)];
        if !q.search.is_empty() {
            // LIKE 默认只对 ASCII 忽略大小写，中文本来就没有大小写，够用了
            sql += " AND text LIKE ? ESCAPE '\\'";
            let s = q.search.replace('\\', "\\\\").replace('%', "\\%").replace('_', "\\_");
            args.push(Box::new(format!("%{s}%")));
        }
        if let Some(k) = q.kind {
            sql += " AND kind = ?";
            args.push(Box::new(k.as_str()));
        }
        if q.pinned {
            sql += " AND pinned = 1";
        }
        sql += " ORDER BY time DESC, id DESC LIMIT ? OFFSET ?";
        args.push(Box::new(q.limit.clamp(1, 1000)));
        args.push(Box::new(q.offset.max(0)));
        let db = self.db.lock().unwrap();
        let mut st = db.prepare(&sql).map_err(err)?;
        let rows = st
            .query_map(rusqlite::params_from_iter(args.iter().map(|a| a.as_ref())), from_row)
            .map_err(err)?;
        rows.collect::<Result<_, _>>().map_err(err)
    }

    /// 取完整的一条
    pub fn get(&self, id: i64) -> Result<Option<Clip>, String> {
        let db = self.db.lock().unwrap();
        db.query_row(&format!("SELECT {COLUMNS_FULL} FROM clips WHERE id = ?"), [id], from_row)
            .optional()
            .map_err(err)
    }

    pub fn set_pinned(&self, id: i64, pinned: bool) -> Result<(), String> {
        let db = self.db.lock().unwrap();
        db.execute("UPDATE clips SET pinned = ? WHERE id = ?", params![pinned, id]).map_err(err)?;
        Ok(())
    }

    /// 用户选了某条重新粘贴：移到最前面
    pub fn touch(&self, id: i64) -> Result<(), String> {
        let db = self.db.lock().unwrap();
        db.execute("UPDATE clips SET time = ? WHERE id = ?", params![now_ms(), id]).map_err(err)?;
        Ok(())
    }

    fn delete_where(&self, cond: &str, args: &[&dyn rusqlite::ToSql]) -> Result<usize, String> {
        let db = self.db.lock().unwrap();
        let hashes: Vec<String> = {
            let mut st = db.prepare(&format!("SELECT hash FROM clips WHERE kind = 'image' AND ({cond})")).map_err(err)?;
            st.query_map(args, |r| r.get(0)).map_err(err)?.collect::<Result<_, _>>().map_err(err)?
        };
        let n = db.execute(&format!("DELETE FROM clips WHERE {cond}"), args).map_err(err)?;
        drop(db);
        for h in hashes {
            let _ = fs::remove_file(self.images.join(format!("{h}.png")));
            let _ = fs::remove_file(self.images.join(format!("{h}.thumb.png")));
        }
        Ok(n)
    }

    pub fn remove(&self, id: i64) -> Result<(), String> {
        self.delete_where("id = ?", &[&id]).map(|_| ())
    }

    /// 清空历史，置顶的保留
    pub fn clear(&self) -> Result<usize, String> {
        self.delete_where("pinned = 0", &[])
    }

    /// 只保留最近 max_items 条，并删掉早于 max_age_days 天的（0 表示不限）。置顶的不删
    pub fn prune(&self, max_items: i64, max_age_days: i64) -> Result<usize, String> {
        let mut n = 0;
        if max_age_days > 0 {
            let before = now_ms() - max_age_days * 86_400_000;
            n += self.delete_where("pinned = 0 AND time < ?", &[&before])?;
        }
        if max_items > 0 {
            n += self.delete_where(
                "pinned = 0 AND id NOT IN (SELECT id FROM clips WHERE pinned = 0 ORDER BY time DESC, id DESC LIMIT ?)",
                &[&max_items],
            )?;
        }
        Ok(n)
    }

    pub fn count(&self) -> Result<i64, String> {
        let db = self.db.lock().unwrap();
        db.query_row("SELECT count(*) FROM clips", [], |r| r.get(0)).map_err(err)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn store() -> (ClipStore, PathBuf) {
        // 测试是并行跑的，每个用自己的目录
        static N: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
        let n = N.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let dir = std::env::temp_dir().join(format!("tooltree-test-{}-{n}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        (ClipStore::open(&dir).unwrap(), dir)
    }

    fn q(search: &str) -> Query {
        Query { search: search.into(), limit: 100, ..Default::default() }
    }

    #[test]
    fn dedup_moves_to_top() {
        let (s, _d) = store();
        let a = s.add_text("hello", "").unwrap().unwrap();
        s.add_text("world", "").unwrap();
        std::thread::sleep(std::time::Duration::from_millis(2));
        let a2 = s.add_text("hello", "Terminal").unwrap().unwrap();
        assert_eq!(a.id, a2.id);
        assert_eq!(a2.source, "Terminal");
        let list = s.list(&q("")).unwrap();
        assert_eq!(list.len(), 2);
        assert_eq!(list[0].text, "hello");
    }

    #[test]
    fn skips_blank_text() {
        let (s, _d) = store();
        assert!(s.add_text("  \n\t", "").unwrap().is_none());
        assert_eq!(s.count().unwrap(), 0);
    }

    #[test]
    fn search_escapes_wildcards() {
        let (s, _d) = store();
        s.add_text("100% done", "").unwrap();
        s.add_text("1000 done", "").unwrap();
        s.add_text("中文内容", "").unwrap();
        assert_eq!(s.list(&q("0%")).unwrap().len(), 1);
        assert_eq!(s.list(&q("DONE")).unwrap().len(), 2);
        assert_eq!(s.list(&q("文内")).unwrap().len(), 1);
    }

    #[test]
    fn prune_keeps_pinned() {
        let (s, _d) = store();
        let first = s.add_text("first", "").unwrap().unwrap();
        s.set_pinned(first.id, true).unwrap();
        for i in 0..10 {
            s.add_text(&format!("item {i}"), "").unwrap();
        }
        s.prune(3, 0).unwrap();
        let list = s.list(&q("")).unwrap();
        assert_eq!(list.len(), 4);
        assert!(list.iter().any(|c| c.id == first.id));
        assert_eq!(s.clear().unwrap(), 3);
        assert_eq!(s.count().unwrap(), 1);
    }

    #[test]
    fn images_saved_and_removed() {
        let (s, _d) = store();
        let rgba = vec![255u8; 800 * 10 * 4];
        let c = s.add_image(800, 10, &rgba, "").unwrap().unwrap();
        assert_eq!((c.width, c.height), (800, 10));
        assert!(s.image_path(&c).exists());
        let thumb = image::open(s.thumb_path(&c)).unwrap();
        assert_eq!(thumb.width(), THUMB_SIZE);
        s.remove(c.id).unwrap();
        assert!(!s.image_path(&c).exists());
        assert!(!s.thumb_path(&c).exists());
    }

    #[test]
    fn preview_is_truncated() {
        let (s, _d) = store();
        let long = "x".repeat(5000);
        let c = s.add_text(&long, "").unwrap().unwrap();
        assert_eq!(c.text.len(), PREVIEW_CHARS as usize);
        assert_eq!(c.size, 5000);
        assert_eq!(s.get(c.id).unwrap().unwrap().text.len(), 5000);
    }
}
