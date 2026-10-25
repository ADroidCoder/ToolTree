import { Binary } from "@lucide/svelte";
import { defineTool } from "../types";

export default defineTool({
  id: "base64",
  name: "Base64",
  desc: "文本和图片的 Base64 编码、解码，支持 URL 安全格式",
  category: "encode",
  icon: Binary,
  keywords: ["base64", "b64", "encode", "decode", "bianma", "jiema", "datauri"],
  order: 1,
  load: () => import("./Tool.svelte"),
});
