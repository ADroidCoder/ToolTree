// 后端命令的封装和数据类型，和 src-tauri/src 里的命令一一对应
import { invoke } from "@tauri-apps/api/core";

export type ClipKind = "text" | "image" | "files";

export interface Clip {
  id: number;
  kind: ClipKind;
  /** 文本内容（列表里只有开头一部分）；文件是每行一个路径；图片为空 */
  text: string;
  /** 文本的字符数、文件个数、图片的字节数 */
  size: number;
  width: number;
  height: number;
  /** 最近一次复制的时间（毫秒） */
  time: number;
  pinned: boolean;
  source: string;
}

export interface ClipQuery {
  search?: string;
  kind?: ClipKind | null;
  pinned?: boolean;
  offset?: number;
  limit?: number;
}

/** 保存在后端的设置（快捷键、剪贴板）。界面外观存在 localStorage */
export interface Config {
  /** 打开工具搜索的全局快捷键，空表示不用 */
  launcherShortcut: string;
  /** 打开剪贴板历史面板的全局快捷键 */
  clipShortcut: string;
  /** 截图的全局快捷键 */
  shotShortcut: string;
  clipEnabled: boolean;
  /** 最多保留多少条（置顶的不算） */
  clipMax: number;
  /** 保留多少天，0 表示不限 */
  clipDays: number;
  /** 在面板里选中后直接粘贴到之前的应用 */
  autoPaste: boolean;
  /** 不记录这些应用里复制的内容（按应用名） */
  ignoreApps: string[];
}

export interface Hashes {
  md5: string;
  sha1: string;
  sha256: string;
  sha384: string;
  sha512: string;
  size: number;
}

export const api = {
  configGet: () => invoke<Config>("config_get"),
  configSet: (config: Config) => invoke<void>("config_set", { config }),

  clipList: (query: ClipQuery) => invoke<Clip[]>("clip_list", { query }),
  clipGet: (id: number) => invoke<Clip | null>("clip_get", { id }),
  clipPin: (id: number, pinned: boolean) => invoke<void>("clip_pin", { id, pinned }),
  clipRemove: (id: number) => invoke<void>("clip_remove", { id }),
  clipClear: () => invoke<number>("clip_clear"),
  clipCount: () => invoke<number>("clip_count"),
  /** 图片的 PNG 数据，thumb 为 true 时是缩略图 */
  clipImage: (id: number, thumb: boolean) => invoke<ArrayBuffer>("clip_image", { id, thumb }),
  /** 写回系统剪贴板 */
  clipCopy: (id: number) => invoke<void>("clip_copy", { id }),
  /** 写回剪贴板，关掉面板，回到之前的应用并粘贴。返回 false 表示没有辅助功能权限，只复制了 */
  clipPaste: (id: number) => invoke<boolean>("clip_paste", { id }),
  /** 把一段文本写进剪贴板（工具里的“复制”按钮） */
  copyText: (text: string) => invoke<void>("copy_text", { text }),

  hashText: (text: string) => invoke<Hashes>("hash_text", { text }),
  hashFile: (path: string) => invoke<Hashes>("hash_file", { path }),

  panelHide: () => invoke<void>("panel_hide"),
  /** 打开主窗口，tool 不为空时切到这个工具，input 作为它的输入 */
  showMain: (tool?: string, input?: string) => invoke<void>("show_main", { tool, input }),
  /** 开始截图；从主窗口点的要先把主窗口藏起来 */
  shotStart: (hideMain: boolean) => invoke<void>("shot_start", { hideMain }),
  openScreenRecordingSettings: () => invoke<void>("open_screen_recording_settings"),
  accessibilityTrusted: () => invoke<boolean>("accessibility_trusted"),
  openAccessibilitySettings: () => invoke<void>("open_accessibility_settings"),
};
