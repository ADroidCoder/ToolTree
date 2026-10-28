// 启动器的图标缓存和使用记录。用得多、最近用过的排得更靠前，搜索框空着时列出最近用过的。

import { api } from "./api";

// ---- 图标 ----

const icons = new Map<string, Promise<string | null>>();

/** 应用、文件的图标地址；取不到（不是 macOS、文件没了）时为 null */
export function iconUrl(path: string): Promise<string | null> {
  let p = icons.get(path);
  if (!p) {
    p = api
      .launcherIcon(path)
      .then((buf) => URL.createObjectURL(new Blob([buf], { type: "image/png" })))
      .catch(() => null);
    icons.set(path, p);
  }
  return p;
}

// ---- 使用记录 ----

/** 记下来的条目，够在“最近使用”里重新显示出来 */
export interface Used {
  kind: "app" | "tool" | "file" | "command";
  /** 应用和文件是路径，工具和命令是 id */
  id: string;
  title: string;
  /** 使用次数 */
  n: number;
  /** 最近一次使用的时间（毫秒） */
  t: number;
}

const KEY = "tooltree.launcher.used";
const MAX = 200;

function load(): Record<string, Used> {
  try {
    return JSON.parse(localStorage.getItem(KEY) ?? "{}") ?? {};
  } catch {
    return {};
  }
}

let used = load();

const key = (kind: Used["kind"], id: string) => `${kind}:${id}`;

export function markUsed(kind: Used["kind"], id: string, title: string) {
  const k = key(kind, id);
  used[k] = { kind, id, title, n: (used[k]?.n ?? 0) + 1, t: Date.now() };
  // 太多了就丢掉最久没用的
  const all = Object.entries(used);
  if (all.length > MAX) used = Object.fromEntries(all.sort((a, b) => b[1].t - a[1].t).slice(0, MAX));
  try {
    localStorage.setItem(KEY, JSON.stringify(used));
  } catch {
    // 存不下就只在这次运行里记着
  }
}

/** 排序加分：用过的次数越多、越近，分越高，最多 20 分 */
export function usageBoost(kind: Used["kind"], id: string) {
  const u = used[key(kind, id)];
  if (!u) return 0;
  const days = (Date.now() - u.t) / 86_400_000;
  return Math.min(14, u.n * 2) + (days < 1 ? 6 : days < 7 ? 3 : 0);
}

export function recent(limit: number): Used[] {
  return Object.values(used)
    .sort((a, b) => b.t - a.t)
    .slice(0, limit);
}

export function forget(kind: Used["kind"], id: string) {
  delete used[key(kind, id)];
  try {
    localStorage.setItem(KEY, JSON.stringify(used));
  } catch {
    // 同上
  }
}

/** 把用户目录换成 ~，路径短一点 */
export function shortPath(path: string, home: string) {
  if (home && (path === home || path.startsWith(home + "/") || path.startsWith(home + "\\"))) return "~" + path.slice(home.length);
  return path;
}
