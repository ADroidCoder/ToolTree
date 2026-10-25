import type { Component } from "svelte";

export type Category = "format" | "encode" | "crypto" | "text" | "time";

/** 侧边栏里分类的顺序和名字 */
export const CATEGORIES: { id: Category; name: string }[] = [
  { id: "format", name: "格式与转换" },
  { id: "encode", name: "编码解码" },
  { id: "crypto", name: "生成与加密" },
  { id: "text", name: "文本" },
  { id: "time", name: "时间" },
];

export interface ToolProps {
  /** 从剪贴板历史、搜索框带过来的输入；为空时用上次的内容 */
  input?: string;
}

/** 每个工具目录下的 index.ts 默认导出一个 ToolMeta，registry.ts 自动收集 */
export interface ToolMeta {
  id: string;
  /** 中文名，界面上用 t() 翻译 */
  name: string;
  desc: string;
  category: Category;
  icon: Component;
  /** 搜索用的别名：英文名、拼音、拼音首字母 */
  keywords: string[];
  /** 同一分类里的顺序 */
  order: number;
  load: () => Promise<{ default: Component<ToolProps> }>;
}

export function defineTool(meta: ToolMeta) {
  return meta;
}
