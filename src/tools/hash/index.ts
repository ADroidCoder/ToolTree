import { Hash } from "@lucide/svelte";
import { defineTool } from "../types";

export default defineTool({
  id: "hash",
  name: "哈希计算",
  desc: "计算文本和文件的 MD5、SHA1、SHA256、SHA512，以及 HMAC",
  category: "crypto",
  icon: Hash,
  keywords: ["hash", "md5", "sha", "sha1", "sha256", "hmac", "checksum", "haxi", "jiaoyan"],
  order: 2,
  load: () => import("./Tool.svelte"),
});
