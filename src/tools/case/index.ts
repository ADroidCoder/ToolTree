import { CaseSensitive } from "@lucide/svelte";
import { defineTool } from "../types";

export default defineTool({
  id: "case",
  name: "命名转换",
  desc: "camelCase、snake_case、kebab-case 等命名风格互相转换，统计字数",
  category: "text",
  icon: CaseSensitive,
  keywords: ["case", "camel", "snake", "kebab", "pascal", "mingming", "daxiaoxie", "zishu"],
  order: 3,
  load: () => import("./Tool.svelte"),
});
