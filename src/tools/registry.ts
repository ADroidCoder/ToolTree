// 自动收集 tools/*/index.ts 里的工具。新增工具只要建一个目录，不用改这里
import { t } from "../lib/i18n.svelte";
import { CATEGORIES, type ToolMeta } from "./types";

const modules = import.meta.glob<{ default: ToolMeta }>("./*/index.ts", { eager: true });

const order = (c: string) => CATEGORIES.findIndex((x) => x.id === c);

export const TOOLS: ToolMeta[] = Object.values(modules)
  .map((m) => m.default)
  .sort((a, b) => order(a.category) - order(b.category) || a.order - b.order);

export function findTool(id: string) {
  return TOOLS.find((x) => x.id === id);
}

/** 按名字、别名搜索工具，越靠前匹配的排得越前 */
export function searchTools(q: string): ToolMeta[] {
  const s = q.trim().toLowerCase();
  if (!s) return TOOLS;
  const scored: [number, ToolMeta][] = [];
  for (const tool of TOOLS) {
    const names = [tool.name, t(tool.name), tool.id, ...tool.keywords].map((x) => x.toLowerCase());
    let best = 0;
    for (const n of names) {
      if (n === s) best = Math.max(best, 4);
      else if (n.startsWith(s)) best = Math.max(best, 3);
      else if (n.includes(s)) best = Math.max(best, 2);
    }
    if (!best && (t(tool.desc).toLowerCase().includes(s) || tool.desc.includes(s))) best = 1;
    if (best) scored.push([best, tool]);
  }
  return scored.sort((a, b) => b[0] - a[0]).map((x) => x[1]);
}
