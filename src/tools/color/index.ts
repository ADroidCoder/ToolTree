import { Palette } from "@lucide/svelte";
import { defineTool } from "../types";

export default defineTool({
  id: "color",
  name: "颜色转换",
  desc: "HEX、RGB、HSL、HSV 互相转换，检查文字对比度",
  category: "format",
  icon: Palette,
  keywords: ["color", "colour", "hex", "rgb", "hsl", "yanse", "ys", "contrast"],
  order: 3,
  load: () => import("./Tool.svelte"),
});
