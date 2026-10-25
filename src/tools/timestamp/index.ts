import { Clock } from "@lucide/svelte";
import { defineTool } from "../types";

export default defineTool({
  id: "timestamp",
  name: "时间戳转换",
  desc: "时间戳和日期互相转换，支持秒、毫秒、微秒和各个时区",
  category: "time",
  icon: Clock,
  keywords: ["timestamp", "time", "date", "unix", "epoch", "shijianchuo", "sjc"],
  order: 1,
  load: () => import("./Tool.svelte"),
});
