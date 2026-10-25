// 全局快捷键的录制和显示。保存的格式就是 global-shortcut 插件认的格式：Alt+Space、CommandOrControl+Shift+V
import { isMac } from "./util";

const MODS = ["Meta", "Control", "Alt", "Shift"];

/** 从按键事件得到快捷键字符串；只按了修饰键或者没有修饰键时返回空 */
export function fromEvent(e: KeyboardEvent): string {
  if (MODS.includes(e.key)) return "";
  const parts: string[] = [];
  if (e.metaKey) parts.push(isMac ? "Command" : "Super");
  if (e.ctrlKey) parts.push("Control");
  if (e.altKey) parts.push("Alt");
  if (e.shiftKey) parts.push("Shift");
  // 单独的功能键可以不带修饰键，其他键至少要一个修饰键，否则会把普通输入都拦掉
  const fn = /^F\d+$/.test(e.code);
  if (!parts.length && !fn) return "";
  const key = e.code.replace(/^Key/, "").replace(/^Digit/, "");
  return [...parts, key].join("+");
}

const SYMBOLS: Record<string, string> = isMac
  ? { CommandOrControl: "⌘", Command: "⌘", Control: "⌃", Alt: "⌥", Shift: "⇧", Super: "⌘", Space: "Space" }
  : { CommandOrControl: "Ctrl", Command: "Win", Control: "Ctrl", Alt: "Alt", Shift: "Shift", Super: "Win", Space: "Space" };

/** 显示用：mac 上 ⌥Space、⌘⇧V，其他系统 Ctrl+Shift+V */
export function display(s: string): string {
  if (!s) return "";
  const parts = s.split("+").map((p) => SYMBOLS[p] ?? p);
  return isMac ? parts.join("") : parts.join("+");
}
