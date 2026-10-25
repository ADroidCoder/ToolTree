import { Calculator } from "@lucide/svelte";
import { defineTool } from "../types";

export default defineTool({
  id: "radix",
  name: "进制转换",
  desc: "二进制、八进制、十进制、十六进制互相转换，支持大数",
  category: "format",
  icon: Calculator,
  keywords: ["radix", "base", "hex", "binary", "octal", "jinzhi", "jz", "erjinzhi", "shiliujinzhi"],
  order: 2,
  load: () => import("./Tool.svelte"),
});
