export interface RGBA {
  r: number;
  g: number;
  b: number;
  a: number;
}

let ctx: CanvasRenderingContext2D | null = null;

/** 解析任何 CSS 认识的颜色（#fff、rgb()、hsl()、red……），交给 canvas 统一换算 */
export function parseColor(input: string): RGBA | null {
  const s = input.trim();
  if (!s) return null;
  // 没写 # 的 6 位或 3 位十六进制也认
  const raw = /^([0-9a-f]{3,4}|[0-9a-f]{6}|[0-9a-f]{8})$/i.test(s) ? "#" + s : s;
  if (!CSS.supports("color", raw)) return null;
  ctx ??= document.createElement("canvas").getContext("2d", { willReadFrequently: true });
  if (!ctx) return null;
  ctx.clearRect(0, 0, 1, 1);
  ctx.fillStyle = "#000";
  ctx.fillStyle = raw;
  ctx.fillRect(0, 0, 1, 1);
  const [r, g, b, a] = ctx.getImageData(0, 0, 1, 1).data;
  return { r, g, b, a: Math.round((a / 255) * 100) / 100 };
}

const h2 = (n: number) => Math.round(n).toString(16).padStart(2, "0");

export function toHex({ r, g, b, a }: RGBA, alpha = false) {
  return "#" + h2(r) + h2(g) + h2(b) + (alpha && a < 1 ? h2(a * 255) : "");
}

export function toHsl({ r, g, b }: RGBA) {
  const [R, G, B] = [r / 255, g / 255, b / 255];
  const max = Math.max(R, G, B);
  const min = Math.min(R, G, B);
  const l = (max + min) / 2;
  let h = 0;
  let s = 0;
  if (max !== min) {
    const d = max - min;
    s = l > 0.5 ? d / (2 - max - min) : d / (max + min);
    h = max === R ? (G - B) / d + (G < B ? 6 : 0) : max === G ? (B - R) / d + 2 : (R - G) / d + 4;
    h *= 60;
  }
  return { h: Math.round(h), s: Math.round(s * 100), l: Math.round(l * 100) };
}

export function toHsv({ r, g, b }: RGBA) {
  const [R, G, B] = [r / 255, g / 255, b / 255];
  const max = Math.max(R, G, B);
  const d = max - Math.min(R, G, B);
  const { h } = toHsl({ r, g, b, a: 1 });
  return { h, s: Math.round(max ? (d / max) * 100 : 0), v: Math.round(max * 100) };
}

/** WCAG 相对亮度和对比度 */
function luminance({ r, g, b }: RGBA) {
  const f = (c: number) => {
    const x = c / 255;
    return x <= 0.03928 ? x / 12.92 : ((x + 0.055) / 1.055) ** 2.4;
  };
  return 0.2126 * f(r) + 0.7152 * f(g) + 0.0722 * f(b);
}

export function contrast(a: RGBA, b: RGBA) {
  const [x, y] = [luminance(a), luminance(b)].sort((m, n) => n - m);
  return (x + 0.05) / (y + 0.05);
}
