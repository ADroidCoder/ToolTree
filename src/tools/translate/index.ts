import { Languages } from "@lucide/svelte";
import { defineTool } from "../types";

export default defineTool({
  id: "translate",
  name: "翻译",
  desc: "中英互译和常用语言翻译，默认免费，也可以用腾讯云、阿里云、DeepL 的密钥",
  category: "text",
  icon: Languages,
  keywords: ["translate", "translation", "fanyi", "fy", "deepl", "google"],
  order: 4,
  load: () => import("./Tool.svelte"),
});
