import { KeyRound } from "@lucide/svelte";
import { defineTool } from "../types";

export default defineTool({
  id: "jwt",
  name: "JWT 解析",
  desc: "解析 JWT 的头部和载荷，查看过期时间，校验 HMAC 签名",
  category: "encode",
  icon: KeyRound,
  keywords: ["jwt", "token", "jws", "bearer", "lingpai"],
  order: 3,
  load: () => import("./Tool.svelte"),
});
