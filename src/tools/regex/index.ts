import { Regex } from "@lucide/svelte";
import { defineTool } from "../types";

export default defineTool({
  id: "regex",
  name: "正则测试",
  desc: "实时测试正则表达式，查看匹配和分组，预览替换结果",
  category: "text",
  icon: Regex,
  keywords: ["regex", "regexp", "zhengze", "zz", "match", "replace"],
  order: 2,
  load: () => import("./Tool.svelte"),
});
