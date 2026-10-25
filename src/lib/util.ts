export const isMac = typeof navigator !== "undefined" && /Mac/.test(navigator.platform);
export const isWindows = typeof navigator !== "undefined" && /Win/.test(navigator.platform);

/** macOS 用 ⌘，其他系统用 Ctrl */
export function modKey(e: KeyboardEvent | MouseEvent) {
  return isMac ? e.metaKey : e.ctrlKey;
}

const pad = (n: number, w = 2) => String(n).padStart(w, "0");

/** 毫秒时间戳 → 2026-10-25 22:08:01 */
export function formatDateTime(ms: number, withMs = false) {
  const d = new Date(ms);
  const s = `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())} ${pad(d.getHours())}:${pad(d.getMinutes())}:${pad(d.getSeconds())}`;
  return withMs ? `${s}.${pad(d.getMilliseconds(), 3)}` : s;
}

export function formatSize(n: number) {
  if (n < 1024) return `${n} B`;
  const units = ["KB", "MB", "GB", "TB"];
  let v = n;
  let i = -1;
  do {
    v /= 1024;
    i++;
  } while (v >= 1024 && i < units.length - 1);
  return `${v < 10 ? v.toFixed(1) : Math.round(v)} ${units[i]}`;
}

export function uid() {
  return Math.random().toString(36).slice(2, 10);
}

/** UTF-8 字符串和字节互转 */
export const utf8 = {
  encode: (s: string) => new TextEncoder().encode(s),
  decode: (b: Uint8Array, fatal = false) => new TextDecoder("utf-8", { fatal }).decode(b),
};

export function bytesToBase64(bytes: Uint8Array) {
  let bin = "";
  for (let i = 0; i < bytes.length; i += 0x8000) bin += String.fromCharCode(...bytes.subarray(i, i + 0x8000));
  return btoa(bin);
}

/** 同时接受标准 Base64 和 URL 安全的 Base64，缺的 = 自动补上 */
export function base64ToBytes(s: string) {
  let b = s.replace(/\s+/g, "").replace(/-/g, "+").replace(/_/g, "/");
  if (b.length % 4) b += "=".repeat(4 - (b.length % 4));
  const bin = atob(b);
  const out = new Uint8Array(bin.length);
  for (let i = 0; i < bin.length; i++) out[i] = bin.charCodeAt(i);
  return out;
}

/** 相对时间：刚刚、3 分钟前、昨天 14:20 */
export function relativeTime(ms: number, t: (s: string, ...a: unknown[]) => string) {
  const diff = Date.now() - ms;
  if (diff < 60_000) return t("刚刚");
  if (diff < 3_600_000) return t("{0} 分钟前", Math.floor(diff / 60_000));
  const d = new Date(ms);
  const today = new Date();
  const hm = `${pad(d.getHours())}:${pad(d.getMinutes())}`;
  if (d.toDateString() === today.toDateString()) return hm;
  const y = new Date(today.getTime() - 86_400_000);
  if (d.toDateString() === y.toDateString()) return t("昨天 {0}", hm);
  if (d.getFullYear() === today.getFullYear()) return `${pad(d.getMonth() + 1)}-${pad(d.getDate())} ${hm}`;
  return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())}`;
}
