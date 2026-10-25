import { Link } from "@lucide/svelte";
import { defineTool } from "../types";

export default defineTool({
  id: "url",
  name: "URL 编解码",
  desc: "URL 编码、解码，拆解网址和查询参数",
  category: "encode",
  icon: Link,
  keywords: ["url", "uri", "encode", "decode", "query", "percent", "wangzhi", "canshu"],
  order: 2,
  load: () => import("./Tool.svelte"),
});
