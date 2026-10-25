import { Fingerprint } from "@lucide/svelte";
import { defineTool } from "../types";

export default defineTool({
  id: "uuid",
  name: "UUID 生成",
  desc: "生成 UUID v4、v7、ULID、NanoID，解析 UUID 的版本和时间",
  category: "crypto",
  icon: Fingerprint,
  keywords: ["uuid", "guid", "ulid", "nanoid", "id", "shengcheng"],
  order: 1,
  load: () => import("./Tool.svelte"),
});
