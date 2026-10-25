import { GitCompare } from "@lucide/svelte";
import { defineTool } from "../types";

export default defineTool({
  id: "diff",
  name: "文本对比",
  desc: "对比两段文本的差异，按行或按字符高亮",
  category: "text",
  icon: GitCompare,
  keywords: ["diff", "compare", "duibi", "bijiao", "chayi"],
  order: 1,
  load: () => import("./Tool.svelte"),
});
