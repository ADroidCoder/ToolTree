// 界面文字的翻译，和 GitTree、ShellTree 的做法一样：中文原文直接当作键，t("复制")，
// 英文界面查 en.ts，查不到就显示原文。参数用 {0} {1} 占位：t("{0} 条记录", n)。
// 英文要区分单复数时写成 "{0} item||{0} items"，按第一个参数是不是 1 来选。

import { invoke } from "@tauri-apps/api/core";
import en from "./en";

export type Lang = "zh" | "en";
const KEY = "tooltree.lang";

function systemLang(): Lang {
  return navigator.language.toLowerCase().startsWith("zh") ? "zh" : "en";
}

/** 用户选的语言；null 表示跟随系统 */
function savedLang(): Lang | null {
  try {
    const v = localStorage.getItem(KEY);
    return v === "zh" || v === "en" ? v : null;
  } catch {
    return null;
  }
}

export const i18n = $state({
  choice: savedLang(),
  lang: savedLang() ?? systemLang(),
});

export function t(text: string, ...args: unknown[]): string {
  let s = (i18n.lang === "en" ? en[text] : undefined) ?? text;
  if (s.includes("||")) {
    const [one, many] = s.split("||");
    s = Number(args[0]) === 1 ? one : many;
  }
  if (!args.length) return s;
  return s.replace(/\{(\d+)\}/g, (m: string, i: string) => (Number(i) < args.length ? String(args[Number(i)]) : m));
}

export function setLang(l: Lang | null) {
  try {
    if (l) localStorage.setItem(KEY, l);
    else localStorage.removeItem(KEY);
  } catch {
    // 存不了就只在这次生效
  }
  i18n.choice = l;
  i18n.lang = l ?? systemLang();
  syncBackendLang();
}

/** 托盘菜单也用同一种语言 */
export function syncBackendLang() {
  return invoke("set_lang", { en: i18n.lang === "en" }).catch(() => {});
}

/** 另一个窗口（剪贴板面板）改了语言，这个窗口跟着变 */
export function watchLang() {
  const on = (e: StorageEvent) => {
    if (e.key !== KEY) return;
    i18n.choice = savedLang();
    i18n.lang = i18n.choice ?? systemLang();
  };
  addEventListener("storage", on);
  return () => removeEventListener("storage", on);
}
