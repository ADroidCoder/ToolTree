import { QrCode } from "@lucide/svelte";
import { defineTool } from "../types";

export default defineTool({
  id: "qr",
  name: "二维码生成",
  desc: "把文本或网址生成二维码，自定义颜色和容错级别，导出 PNG、SVG",
  category: "crypto",
  icon: QrCode,
  keywords: ["qr", "qrcode", "qr code", "erweima", "ewm", "shengcheng"],
  order: 3,
  load: () => import("./Tool.svelte"),
});
