// 外观设置，存在 localStorage 里。主窗口和剪贴板面板共用，一个窗口改了另一个跟着变
import { getCurrentWindow } from "@tauri-apps/api/window";
export type Theme = "dark" | "light" | "system";

const THEME_KEY = "tooltree.theme";

function read(key: string, def: string) {
  try {
    return localStorage.getItem(key) ?? def;
  } catch {
    return def;
  }
}

export const settings = $state({
  theme: read(THEME_KEY, "dark") as Theme,
  /** 系统是不是深色，主题为“跟随系统”时用 */
  systemDark: typeof matchMedia !== "undefined" && matchMedia("(prefers-color-scheme: dark)").matches,
});

export function isDark() {
  return settings.theme === "dark" || (settings.theme === "system" && settings.systemDark);
}

export function setTheme(t: Theme) {
  settings.theme = t;
  try {
    localStorage.setItem(THEME_KEY, t);
  } catch {
    // 存不了就只在这次生效
  }
  applyTheme();
}

/** 页面配色和窗口外观一起切换。窗口外观决定 macOS 红绿灯按钮的颜色 */
export function applyTheme() {
  document.documentElement.dataset.theme = settings.theme;
  getCurrentWindow()
    .setTheme(settings.theme === "system" ? null : settings.theme)
    .catch(() => {});
}

export function watchSystemTheme() {
  const mq = matchMedia("(prefers-color-scheme: dark)");
  const onMq = () => (settings.systemDark = mq.matches);
  const onStorage = (e: StorageEvent) => {
    if (e.key !== THEME_KEY) return;
    settings.theme = read(THEME_KEY, "dark") as Theme;
    applyTheme();
  };
  mq.addEventListener("change", onMq);
  addEventListener("storage", onStorage);
  return () => {
    mq.removeEventListener("change", onMq);
    removeEventListener("storage", onStorage);
  };
}
