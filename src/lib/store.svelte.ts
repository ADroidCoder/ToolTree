// 主窗口的界面状态：当前页面、打开的工具、收藏和最近使用、提示消息
import { api } from "./api";
import { t } from "./i18n.svelte";
import { uid } from "./util";

export type Page = "home" | "tool" | "clipboard" | "settings";

interface Toast {
  id: string;
  text: string;
  kind: "info" | "error";
}

function readList(key: string): string[] {
  try {
    const v = JSON.parse(localStorage.getItem(key) ?? "[]");
    return Array.isArray(v) ? v : [];
  } catch {
    return [];
  }
}

function writeList(key: string, v: string[]) {
  try {
    localStorage.setItem(key, JSON.stringify(v));
  } catch {
    // 存不了就只在这次生效
  }
}

export const app = $state({
  page: "home" as Page,
  tool: "",
  /** 交给工具的输入。每次打开换一个 key，同一个工具也会重新加载这段输入 */
  handoff: null as { tool: string; input: string; key: string } | null,
  palette: false,
  favorites: readList("tooltree.favorites"),
  recents: readList("tooltree.recents"),
  toasts: [] as Toast[],
});

export function openTool(id: string, input?: string) {
  app.page = "tool";
  app.tool = id;
  app.handoff = input === undefined ? null : { tool: id, input, key: uid() };
  app.recents = [id, ...app.recents.filter((x) => x !== id)].slice(0, 8);
  writeList("tooltree.recents", app.recents);
}

export function openPage(p: Exclude<Page, "tool">) {
  app.page = p;
}

export function toggleFavorite(id: string) {
  app.favorites = app.favorites.includes(id) ? app.favorites.filter((x) => x !== id) : [...app.favorites, id];
  writeList("tooltree.favorites", app.favorites);
}

export function toast(text: string, kind: Toast["kind"] = "info") {
  const id = uid();
  app.toasts.push({ id, text, kind });
  setTimeout(() => (app.toasts = app.toasts.filter((x) => x.id !== id)), kind === "error" ? 5000 : 1800);
}

export async function copy(text: string) {
  if (!text) return;
  try {
    await api.copyText(text);
    toast(t("已复制"));
  } catch (e) {
    toast(String(e), "error");
  }
}
