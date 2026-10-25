// 截图标注的形状和绘制。坐标都是截图窗口里的 CSS 像素；
// 屏幕上显示和导出成图片用同一套绘制函数，只是画布的缩放和偏移不同，所以看到的就是得到的。

export type Tool = "rect" | "ellipse" | "arrow" | "pen" | "text" | "mosaic" | "marker";

export interface Rect {
  x: number;
  y: number;
  w: number;
  h: number;
}

/** 两点确定的形状：矩形、椭圆、箭头、马赛克 */
export interface BoxShape {
  kind: "rect" | "ellipse" | "arrow" | "mosaic";
  x1: number;
  y1: number;
  x2: number;
  y2: number;
  color: string;
  size: number;
}

export interface PenShape {
  kind: "pen";
  points: number[];
  color: string;
  size: number;
}

export interface TextShape {
  kind: "text";
  x: number;
  y: number;
  text: string;
  color: string;
  size: number;
}

/** 带序号的圆点，用来标注步骤 */
export interface MarkerShape {
  kind: "marker";
  x: number;
  y: number;
  n: number;
  color: string;
  size: number;
}

export type Shape = BoxShape | PenShape | TextShape | MarkerShape;

/** 截到的屏幕画面。scale 是图片像素和 CSS 像素的比例（Retina 屏上是 2） */
export interface Source {
  bitmap: ImageBitmap;
  scale: number;
}

export function normRect(x1: number, y1: number, x2: number, y2: number): Rect {
  return { x: Math.min(x1, x2), y: Math.min(y1, y2), w: Math.abs(x2 - x1), h: Math.abs(y2 - y1) };
}

/** 线宽对应的字号和序号圆点半径 */
export const fontSize = (size: number) => 12 + size * 3;
export const markerRadius = (size: number) => 9 + size * 1.5;
export const FONT = '-apple-system, BlinkMacSystemFont, "Segoe UI", "PingFang SC", "Microsoft YaHei", sans-serif';

// 马赛克的缩小图按形状缓存，拖动选区、重画时不用每次重新采样
const mosaicCache = new WeakMap<BoxShape, { key: string; canvas: OffscreenCanvas }>();

function drawMosaic(ctx: CanvasRenderingContext2D | OffscreenCanvasRenderingContext2D, s: BoxShape, src: Source) {
  const r = normRect(s.x1, s.y1, s.x2, s.y2);
  if (r.w < 1 || r.h < 1) return;
  const block = Math.max(6, s.size * 3);
  const tw = Math.max(1, Math.ceil(r.w / block));
  const th = Math.max(1, Math.ceil(r.h / block));
  const key = `${r.x},${r.y},${r.w},${r.h},${block}`;
  let cached = mosaicCache.get(s);
  if (!cached || cached.key !== key) {
    const canvas = new OffscreenCanvas(tw, th);
    const c = canvas.getContext("2d")!;
    c.drawImage(src.bitmap, r.x * src.scale, r.y * src.scale, r.w * src.scale, r.h * src.scale, 0, 0, tw, th);
    cached = { key, canvas };
    mosaicCache.set(s, cached);
  }
  ctx.save();
  ctx.imageSmoothingEnabled = false;
  ctx.drawImage(cached.canvas, 0, 0, tw, th, r.x, r.y, tw * block, th * block);
  ctx.restore();
}

function drawArrow(ctx: CanvasRenderingContext2D | OffscreenCanvasRenderingContext2D, s: BoxShape) {
  const dx = s.x2 - s.x1;
  const dy = s.y2 - s.y1;
  const len = Math.hypot(dx, dy);
  if (len < 2) return;
  const head = Math.min(len * 0.6, Math.max(12, s.size * 4.5));
  const half = head * 0.45;
  const ux = dx / len;
  const uy = dy / len;
  // 箭头根部
  const bx = s.x2 - ux * head;
  const by = s.y2 - uy * head;
  ctx.beginPath();
  ctx.moveTo(s.x1, s.y1);
  ctx.lineTo(bx + ux * 1, by + uy * 1);
  ctx.lineWidth = s.size;
  ctx.lineCap = "round";
  ctx.stroke();
  ctx.beginPath();
  ctx.moveTo(s.x2, s.y2);
  ctx.lineTo(bx - uy * half, by + ux * half);
  ctx.lineTo(bx + uy * half, by - ux * half);
  ctx.closePath();
  ctx.fill();
}

export function drawShape(ctx: CanvasRenderingContext2D | OffscreenCanvasRenderingContext2D, s: Shape, src: Source) {
  ctx.save();
  ctx.strokeStyle = s.color;
  ctx.fillStyle = s.color;
  ctx.lineJoin = "round";
  switch (s.kind) {
    case "rect": {
      const r = normRect(s.x1, s.y1, s.x2, s.y2);
      ctx.lineWidth = s.size;
      ctx.strokeRect(r.x, r.y, r.w, r.h);
      break;
    }
    case "ellipse": {
      const r = normRect(s.x1, s.y1, s.x2, s.y2);
      ctx.lineWidth = s.size;
      ctx.beginPath();
      ctx.ellipse(r.x + r.w / 2, r.y + r.h / 2, r.w / 2, r.h / 2, 0, 0, Math.PI * 2);
      ctx.stroke();
      break;
    }
    case "arrow":
      drawArrow(ctx, s);
      break;
    case "mosaic":
      drawMosaic(ctx, s, src);
      break;
    case "pen": {
      const p = s.points;
      if (p.length < 2) break;
      ctx.lineWidth = s.size;
      ctx.lineCap = "round";
      ctx.beginPath();
      ctx.moveTo(p[0], p[1]);
      // 用中点做二次曲线，手画的线更平滑
      for (let i = 2; i < p.length - 2; i += 2) {
        const mx = (p[i] + p[i + 2]) / 2;
        const my = (p[i + 1] + p[i + 3]) / 2;
        ctx.quadraticCurveTo(p[i], p[i + 1], mx, my);
      }
      ctx.lineTo(p[p.length - 2], p[p.length - 1]);
      ctx.stroke();
      break;
    }
    case "text": {
      const fs = fontSize(s.size);
      ctx.font = `600 ${fs}px ${FONT}`;
      ctx.textBaseline = "top";
      // 细描边，在任何背景上都看得清
      ctx.lineWidth = Math.max(2, fs / 8);
      ctx.strokeStyle = isLight(s.color) ? "rgba(0,0,0,0.55)" : "rgba(255,255,255,0.75)";
      s.text.split("\n").forEach((line, i) => {
        ctx.strokeText(line, s.x, s.y + i * fs * 1.25);
        ctx.fillText(line, s.x, s.y + i * fs * 1.25);
      });
      break;
    }
    case "marker": {
      const r = markerRadius(s.size);
      ctx.beginPath();
      ctx.arc(s.x, s.y, r, 0, Math.PI * 2);
      ctx.fill();
      ctx.lineWidth = 2;
      ctx.strokeStyle = "#fff";
      ctx.stroke();
      ctx.fillStyle = isLight(s.color) ? "#000" : "#fff";
      ctx.font = `700 ${Math.round(r * 1.15)}px ${FONT}`;
      ctx.textAlign = "center";
      ctx.textBaseline = "middle";
      ctx.fillText(String(s.n), s.x, s.y + r * 0.05);
      break;
    }
  }
  ctx.restore();
}

export function isLight(color: string) {
  const m = /^#([0-9a-f]{2})([0-9a-f]{2})([0-9a-f]{2})$/i.exec(color);
  if (!m) return false;
  const [r, g, b] = m.slice(1).map((h) => parseInt(h, 16));
  return 0.299 * r + 0.587 * g + 0.114 * b > 170;
}

/** 形状太小（手抖点了一下）就不要 */
export function isTrivial(s: Shape) {
  if (s.kind === "pen") return s.points.length < 4;
  if (s.kind === "text") return !s.text.trim();
  if (s.kind === "marker") return false;
  return Math.hypot(s.x2 - s.x1, s.y2 - s.y1) < 3;
}

/** 导出选区：截图加标注，按原始分辨率画，返回 8 字节宽高开头的 RGBA */
export function exportRegion(src: Source, sel: Rect, shapes: Shape[], screen: { w: number; h: number }): Uint8Array {
  const s = src.scale;
  const W = Math.max(1, Math.round(sel.w * s));
  const H = Math.max(1, Math.round(sel.h * s));
  const canvas = new OffscreenCanvas(W, H);
  const ctx = canvas.getContext("2d")!;
  ctx.setTransform(s, 0, 0, s, -sel.x * s, -sel.y * s);
  ctx.drawImage(src.bitmap, 0, 0, screen.w, screen.h);
  for (const sh of shapes) drawShape(ctx, sh, src);
  const data = ctx.getImageData(0, 0, W, H).data;
  const out = new Uint8Array(8 + data.length);
  const view = new DataView(out.buffer);
  view.setUint32(0, W, true);
  view.setUint32(4, H, true);
  out.set(data, 8);
  return out;
}

/** 后端发来的 8 字节宽高开头的 RGBA 转成可以画的图 */
export async function decodeRaw(buf: ArrayBuffer) {
  const view = new DataView(buf);
  const w = view.getUint32(0, true);
  const h = view.getUint32(4, true);
  const pixels = new Uint8ClampedArray(buf, 8, w * h * 4);
  const bitmap = await createImageBitmap(new ImageData(pixels, w, h));
  return { w, h, pixels, bitmap };
}
