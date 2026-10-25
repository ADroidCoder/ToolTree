import { Braces } from "@lucide/svelte";
import { defineTool } from "../types";

export default defineTool({
  id: "json",
  name: "JSON 工具",
  desc: "格式化、压缩、校验，转换成 YAML 和 TypeScript 类型",
  category: "format",
  icon: Braces,
  keywords: ["json", "format", "yaml", "typescript", "geshihua", "gsh"],
  order: 1,
  load: () => import("./Tool.svelte"),
});
