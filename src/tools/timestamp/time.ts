// 把各种输入统一成毫秒时间戳

export type Unit = "s" | "ms" | "us" | "ns";

/** 纯数字按位数猜单位：10 位秒，13 位毫秒，16 位微秒，19 位纳秒 */
export function guessUnit(digits: string): Unit {
  const n = digits.replace(/^-/, "").length;
  if (n <= 11) return "s";
  if (n <= 14) return "ms";
  if (n <= 17) return "us";
  return "ns";
}

export function parseInput(raw: string): { ms: number; unit?: Unit } | null {
  const s = raw.trim();
  if (!s) return null;
  if (/^-?\d+(\.\d+)?$/.test(s)) {
    const unit = guessUnit(s.split(".")[0]);
    const n = Number(s);
    const ms = unit === "s" ? n * 1000 : unit === "ms" ? n : unit === "us" ? n / 1000 : n / 1e6;
    return Number.isFinite(ms) ? { ms: Math.trunc(ms), unit } : null;
  }
  // 2026/1/5 12:00 这种写法统一成 2026-01-05T12:00，Safari 内核只认这种
  const normalized = s
    .replace(/^(\d{4})[/.](\d{1,2})[/.](\d{1,2})/, "$1-$2-$3")
    .replace(/^(\d{4})-(\d)(?=-)/, "$1-0$2")
    .replace(/^(\d{4}-\d{2})-(\d)(?!\d)/, "$1-0$2")
    .replace(/^(\d{4}-\d{2}-\d{2}) (?=\d)/, "$1T");
  for (const c of [normalized, s]) {
    const ms = Date.parse(c);
    if (!Number.isNaN(ms)) return { ms };
  }
  return null;
}

/** 3 天后、2 小时前 */
export function relativeOffset(ms: number, now: number, t: (s: string, ...a: unknown[]) => string) {
  const diff = ms - now;
  const abs = Math.abs(diff);
  const units: [number, string][] = [
    [365 * 86_400_000, "年"],
    [30 * 86_400_000, "个月"],
    [86_400_000, "天"],
    [3_600_000, "小时"],
    [60_000, "分钟"],
    [1000, "秒"],
  ];
  for (const [size, name] of units) {
    if (abs >= size) {
      const n = Math.floor(abs / size);
      return diff > 0 ? t("{0} {1}后", n, t(name)) : t("{0} {1}前", n, t(name));
    }
  }
  return t("现在");
}

/** 在指定时区格式化成 2026-10-25 22:08:01.123 */
export function formatInZone(ms: number, timeZone: string, withMs = true) {
  const parts = new Intl.DateTimeFormat("en-CA", {
    timeZone,
    year: "numeric",
    month: "2-digit",
    day: "2-digit",
    hour: "2-digit",
    minute: "2-digit",
    second: "2-digit",
    hourCycle: "h23",
  }).formatToParts(new Date(ms));
  const p = Object.fromEntries(parts.map((x) => [x.type, x.value]));
  const base = `${p.year}-${p.month}-${p.day} ${p.hour}:${p.minute}:${p.second}`;
  return withMs ? `${base}.${String(((ms % 1000) + 1000) % 1000).padStart(3, "0")}` : base;
}

/** 时区的 UTC 偏移，比如 +08:00 */
export function zoneOffset(ms: number, timeZone: string) {
  const name = new Intl.DateTimeFormat("en-US", { timeZone, timeZoneName: "longOffset" })
    .formatToParts(new Date(ms))
    .find((x) => x.type === "timeZoneName")?.value;
  const off = name?.replace("GMT", "") || "+00:00";
  return off;
}
