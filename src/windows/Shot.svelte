<!-- 截图窗口：盖在一个屏幕上，显示截到的画面。
     先选区域（悬停自动吸附窗口，拖动自由框选），再标注，最后复制、保存或钉在桌面上。 -->
<script lang="ts">
  import { onMount, tick } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import {
    ArrowUpRight, Check, Circle, Download, Grid3x3, Pencil, Pin, Square, Type, Undo2, X, Hash as HashIcon,
  } from "@lucide/svelte";
  import { t, watchLang } from "../lib/i18n.svelte";
  import { api } from "../lib/api";
  import { modKey } from "../lib/util";
  import {
    decodeRaw, drawShape, exportRegion, fontSize, FONT, isTrivial, normRect,
    type Rect, type Shape, type Source, type Tool,
  } from "../lib/shot/draw";

  interface Info {
    id: number;
    width: number;
    height: number;
    windows: Rect[];
    active: boolean;
  }

  const COLORS = ["#ff3b30", "#ffcc00", "#34c759", "#0a84ff", "#ffffff", "#000000"];
  const SIZES = [2, 4, 8];
  const TOOLS: { id: Tool; icon: typeof Square; name: string; key: string }[] = [
    { id: "rect", icon: Square, name: "矩形", key: "1" },
    { id: "ellipse", icon: Circle, name: "椭圆", key: "2" },
    { id: "arrow", icon: ArrowUpRight, name: "箭头", key: "3" },
    { id: "pen", icon: Pencil, name: "画笔", key: "4" },
    { id: "text", icon: Type, name: "文字", key: "5" },
    { id: "mosaic", icon: Grid3x3, name: "马赛克", key: "6" },
    { id: "marker", icon: HashIcon, name: "序号", key: "7" },
  ];

  let canvas = $state<HTMLCanvasElement>();
  let info: Info | null = null;
  let src = $state.raw<Source | null>(null);
  let pixels: Uint8ClampedArray | null = null;
  let imgW = 0;

  let W = $state(0);
  let H = $state(0);
  /** 还没选区域时，鼠标下面的窗口（或整个屏幕） */
  let hover = $state<Rect | null>(null);
  let sel = $state<Rect | null>(null);
  let shapes = $state<Shape[]>([]);
  let draft = $state<Shape | null>(null);
  let tool = $state<Tool | null>(null);
  let color = $state(COLORS[0]);
  let size = $state(SIZES[1]);
  let mouse = $state({ x: -1, y: -1 });
  let busy = false;

  /** 正在输入的文字 */
  let editing = $state<{ x: number; y: number; text: string } | null>(null);
  let textEl = $state<HTMLTextAreaElement>();

  type Drag =
    | { kind: "select"; x: number; y: number; moved: boolean }
    | { kind: "move"; dx: number; dy: number }
    | { kind: "resize"; handle: string; start: Rect; x: number; y: number }
    | { kind: "draw" };
  let drag = $state.raw<Drag | null>(null);

  // ---- 加载 ----

  async function load() {
    const next = await invoke<Info | null>("shot_info");
    if (!next || next.id === info?.id) return;
    const buf = await invoke<ArrayBuffer>("shot_image");
    const img = await decodeRaw(buf);
    src?.bitmap.close();
    info = next;
    W = next.width;
    H = next.height;
    imgW = img.w;
    pixels = img.pixels;
    src = { bitmap: img.bitmap, scale: img.w / next.width };
    reset();
    await tick();
    resizeCanvas();
    render();
    // 画好了再让后端显示窗口，不会闪出上一次的画面
    await invoke("shot_ready");
    if (next.active) window.focus();
  }

  function reset() {
    sel = null;
    shapes = [];
    draft = null;
    tool = null;
    editing = null;
    drag = null;
    hover = null;
    busy = false;
  }

  function resizeCanvas() {
    if (!canvas) return;
    const k = devicePixelRatio || 1;
    canvas.width = Math.round(W * k);
    canvas.height = Math.round(H * k);
  }

  onMount(() => {
    const stopLang = watchLang();
    const un = listen("shot-start", load);
    load();
    return () => {
      stopLang();
      un.then((f) => f());
    };
  });

  // ---- 绘制 ----

  let frame = 0;
  function render() {
    cancelAnimationFrame(frame);
    frame = requestAnimationFrame(paint);
  }

  function paint() {
    const ctx = canvas?.getContext("2d");
    if (!ctx || !src) return;
    const k = canvas!.width / W;
    ctx.setTransform(k, 0, 0, k, 0, 0);
    ctx.drawImage(src.bitmap, 0, 0, W, H);
    const focus = sel ?? hover;
    // 选区以外压暗
    ctx.fillStyle = "rgba(0, 0, 0, 0.5)";
    if (focus) {
      ctx.fillRect(0, 0, W, focus.y);
      ctx.fillRect(0, focus.y + focus.h, W, H - focus.y - focus.h);
      ctx.fillRect(0, focus.y, focus.x, focus.h);
      ctx.fillRect(focus.x + focus.w, focus.y, W - focus.x - focus.w, focus.h);
    } else {
      ctx.fillRect(0, 0, W, H);
    }
    if (sel) {
      ctx.save();
      ctx.beginPath();
      ctx.rect(sel.x, sel.y, sel.w, sel.h);
      ctx.clip();
      for (const s of shapes) drawShape(ctx, s, src);
      if (draft) drawShape(ctx, draft, src);
      ctx.restore();
    }
    if (focus) {
      ctx.strokeStyle = "#2f8cff";
      ctx.lineWidth = sel ? 1.5 : 3;
      ctx.strokeRect(focus.x, focus.y, focus.w, focus.h);
    }
  }

  $effect(() => {
    // 这些变化时重画
    void [sel, hover, shapes, draft, W, H];
    render();
  });

  // ---- 放大镜 ----

  const ZOOM = 8;
  const LENS = 15; // 放大镜里显示的像素数（奇数，中间那个是鼠标所在的点）
  let lens = $state<HTMLCanvasElement>();

  function pixelAt(x: number, y: number) {
    if (!pixels || !src) return null;
    const px = Math.min(imgW - 1, Math.max(0, Math.floor(x * src.scale)));
    const py = Math.max(0, Math.floor(y * src.scale));
    const i = (py * imgW + px) * 4;
    if (i + 3 >= pixels.length) return null;
    return [pixels[i], pixels[i + 1], pixels[i + 2]];
  }

  const hex = $derived.by(() => {
    const p = pixelAt(mouse.x, mouse.y);
    return p ? "#" + p.map((v) => v.toString(16).padStart(2, "0")).join("").toUpperCase() : "";
  });

  const showLens = $derived(mouse.x >= 0 && (!sel || drag?.kind === "select" || drag?.kind === "resize"));

  $effect(() => {
    if (!showLens || !lens || !src) return;
    const ctx = lens.getContext("2d")!;
    const s = src.scale;
    const n = LENS;
    ctx.imageSmoothingEnabled = false;
    ctx.fillStyle = "#000";
    ctx.fillRect(0, 0, lens.width, lens.height);
    const cx = Math.floor(mouse.x * s);
    const cy = Math.floor(mouse.y * s);
    ctx.drawImage(src.bitmap, cx - (n >> 1), cy - (n >> 1), n, n, 0, 0, n * ZOOM, n * ZOOM);
    // 中心十字
    ctx.fillStyle = "rgba(47, 140, 255, 0.35)";
    ctx.fillRect(0, (n >> 1) * ZOOM, n * ZOOM, ZOOM);
    ctx.fillRect((n >> 1) * ZOOM, 0, ZOOM, n * ZOOM);
  });

  const lensPos = $derived.by(() => {
    const w = LENS * ZOOM + 2;
    const h = w + 44;
    let x = mouse.x + 20;
    let y = mouse.y + 20;
    if (x + w > W) x = mouse.x - 20 - w;
    if (y + h > H) y = mouse.y - 20 - h;
    return { x, y };
  });

  // ---- 鼠标 ----

  const clamp = (v: number, lo: number, hi: number) => Math.max(lo, Math.min(hi, v));

  function windowAt(x: number, y: number): Rect {
    const w = info?.windows.find((r) => x >= r.x && x < r.x + r.w && y >= r.y && y < r.y + r.h);
    return w ?? { x: 0, y: 0, w: W, h: H };
  }

  function inSel(x: number, y: number) {
    return !!sel && x >= sel.x && x <= sel.x + sel.w && y >= sel.y && y <= sel.y + sel.h;
  }

  function onDown(e: MouseEvent) {
    if (e.button !== 0) return;
    // 不让浏览器在按下时把焦点移到别处，否则刚放好的文字输入框会马上失去焦点
    e.preventDefault();
    const x = e.clientX;
    const y = e.clientY;
    if (editing) {
      commitText();
      return;
    }
    if (!sel) {
      drag = { kind: "select", x, y, moved: false };
      return;
    }
    if (!inSel(x, y)) return;
    if (!tool) {
      drag = { kind: "move", dx: x - sel.x, dy: y - sel.y };
      return;
    }
    if (tool === "text") {
      editing = { x, y: y - fontSize(size) / 2, text: "" };
      tick().then(() => textEl?.focus());
      return;
    }
    if (tool === "marker") {
      const n = shapes.filter((s) => s.kind === "marker").length + 1;
      shapes = [...shapes, { kind: "marker", x, y, n, color, size }];
      return;
    }
    draft = tool === "pen" ? { kind: "pen", points: [x, y], color, size } : { kind: tool, x1: x, y1: y, x2: x, y2: y, color, size };
    drag = { kind: "draw" };
  }

  function onMove(e: MouseEvent) {
    const x = clamp(e.clientX, 0, W);
    const y = clamp(e.clientY, 0, H);
    mouse = { x, y };
    if (!drag) {
      if (!sel) hover = windowAt(x, y);
      return;
    }
    if (drag.kind === "select") {
      if (!drag.moved && Math.hypot(x - drag.x, y - drag.y) < 4) return;
      drag.moved = true;
      hover = null;
      sel = normRect(drag.x, drag.y, x, y);
    } else if (drag.kind === "move" && sel) {
      sel = { ...sel, x: clamp(x - drag.dx, 0, W - sel.w), y: clamp(y - drag.dy, 0, H - sel.h) };
    } else if (drag.kind === "resize") {
      const r = drag.start;
      let x1 = r.x, y1 = r.y, x2 = r.x + r.w, y2 = r.y + r.h;
      if (drag.handle.includes("w")) x1 = x;
      if (drag.handle.includes("e")) x2 = x;
      if (drag.handle.includes("n")) y1 = y;
      if (drag.handle.includes("s")) y2 = y;
      sel = normRect(x1, y1, x2, y2);
    } else if (drag.kind === "draw" && draft && sel) {
      // 画的时候限制在选区里
      const cx = clamp(x, sel.x, sel.x + sel.w);
      const cy = clamp(y, sel.y, sel.y + sel.h);
      if (draft.kind === "pen") draft = { ...draft, points: [...draft.points, cx, cy] };
      else if (draft.kind !== "text" && draft.kind !== "marker") {
        let x2 = cx, y2 = cy;
        // 按住 Shift 画正方形、正圆、水平或垂直的箭头
        if (e.shiftKey) {
          if (draft.kind === "arrow") {
            if (Math.abs(cx - draft.x1) > Math.abs(cy - draft.y1)) y2 = draft.y1;
            else x2 = draft.x1;
          } else {
            const d = Math.max(Math.abs(cx - draft.x1), Math.abs(cy - draft.y1));
            x2 = draft.x1 + Math.sign(cx - draft.x1 || 1) * d;
            y2 = draft.y1 + Math.sign(cy - draft.y1 || 1) * d;
          }
        }
        draft = { ...draft, x2, y2 };
      }
    }
  }

  function onUp() {
    if (!drag) return;
    if (drag.kind === "select" && !drag.moved) sel = hover ?? { x: 0, y: 0, w: W, h: H };
    if (drag.kind === "draw" && draft) {
      if (!isTrivial(draft)) shapes = [...shapes, draft];
      draft = null;
    }
    if (sel && (sel.w < 2 || sel.h < 2)) sel = null;
    drag = null;
  }

  function startResize(e: MouseEvent, handle: string) {
    if (e.button !== 0 || !sel) return;
    e.stopPropagation();
    drag = { kind: "resize", handle, start: { ...sel }, x: e.clientX, y: e.clientY };
  }

  /** 右键：已经选了区域就回到重新选择，没选就退出 */
  function onContext(e: MouseEvent) {
    e.preventDefault();
    if (editing) editing = null;
    else if (sel) {
      sel = null;
      shapes = [];
      tool = null;
      hover = windowAt(e.clientX, e.clientY);
    } else cancel();
  }

  function onDouble(e: MouseEvent) {
    if (sel && !tool && inSel(e.clientX, e.clientY)) finish("copy");
  }

  // ---- 文字 ----

  function commitText() {
    if (editing && editing.text.trim()) {
      shapes = [...shapes, { kind: "text", x: editing.x, y: editing.y, text: editing.text, color, size }];
    }
    editing = null;
  }

  // ---- 完成 ----

  async function finish(action: "copy" | "save" | "pin") {
    if (!sel || !src || !info || busy) return;
    commitText();
    busy = true;
    const data = exportRegion(src, sel, shapes, { w: W, h: H });
    const headers = { action, x: String(sel.x), y: String(sel.y), w: String(sel.w), h: String(sel.h) };
    try {
      await invoke("shot_finish", data, { headers });
    } finally {
      reset();
      info = null;
    }
  }

  function cancel() {
    reset();
    info = null;
    invoke("shot_cancel");
  }

  function undo() {
    shapes = shapes.slice(0, -1);
  }

  function onKey(e: KeyboardEvent) {
    if (editing) {
      if (e.key === "Escape") editing = null;
      else if (e.key === "Enter" && !e.shiftKey) commitText();
      else return;
      e.preventDefault();
      return;
    }
    const k = e.key.toLowerCase();
    if (e.key === "Escape") cancel();
    else if (e.key === "Enter" || (modKey(e) && k === "c")) finish("copy");
    else if (modKey(e) && k === "s") finish("save");
    else if (modKey(e) && k === "z") undo();
    else if (e.key === "F3" || (modKey(e) && k === "t")) finish("pin");
    // 没选区域时按 C 复制鼠标位置的颜色
    else if (!sel && k === "c" && hex) {
      api.copyText(hex).finally(cancel);
    } else if (sel && !modKey(e) && /^[1-7]$/.test(e.key)) {
      const tl = TOOLS[Number(e.key) - 1].id;
      tool = tool === tl ? null : tl;
    } else if (sel && e.key.startsWith("Arrow")) {
      // 方向键微调选区，按住 Shift 一次 10 像素
      const d = e.shiftKey ? 10 : 1;
      const dx = e.key === "ArrowLeft" ? -d : e.key === "ArrowRight" ? d : 0;
      const dy = e.key === "ArrowUp" ? -d : e.key === "ArrowDown" ? d : 0;
      sel = { ...sel, x: clamp(sel.x + dx, 0, W - sel.w), y: clamp(sel.y + dy, 0, H - sel.h) };
    } else return;
    e.preventDefault();
  }

  // ---- 工具栏位置：选区下方，放不下就放上方，再不行放在选区里 ----

  const BAR_H = 40;
  const barPos = $derived.by(() => {
    if (!sel) return null;
    const right = Math.min(W - 8, sel.x + sel.w);
    let top = sel.y + sel.h + 8;
    if (top + BAR_H * 2 > H) top = sel.y - BAR_H * 2 - 8;
    if (top < 0) top = sel.y + sel.h - BAR_H * 2 - 8;
    return { right: W - right, top };
  });

  const sizeLabel = $derived(sel && src ? `${Math.round(sel.w * src.scale)} × ${Math.round(sel.h * src.scale)}` : "");
  const HANDLES = ["nw", "n", "ne", "e", "se", "s", "sw", "w"];
</script>

<svelte:window onkeydown={onKey} onmousemove={onMove} onmouseup={onUp} />

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
  class="stage"
  class:pick={!sel}
  class:movable={sel && !tool}
  class:drawing={sel && tool}
  onmousedown={onDown}
  oncontextmenu={onContext}
  ondblclick={onDouble}
>
  <canvas bind:this={canvas} style:width="{W}px" style:height="{H}px"></canvas>

  {#if sel}
    <div class="size" style:left="{sel.x}px" style:top="{Math.max(0, sel.y - 26)}px">{sizeLabel}</div>
    {#if !tool}
      {#each HANDLES as h (h)}
        <!-- svelte-ignore a11y_no_static_element_interactions -->
        <div
          class="handle {h}"
          style:left="{sel.x + (h.includes('w') ? 0 : h.includes('e') ? sel.w : sel.w / 2)}px"
          style:top="{sel.y + (h.includes('n') ? 0 : h.includes('s') ? sel.h : sel.h / 2)}px"
          onmousedown={(e) => startResize(e, h)}
        ></div>
      {/each}
    {/if}
  {/if}

  {#if editing}
    <textarea
      class="text-input"
      bind:this={textEl}
      bind:value={editing.text}
      onmousedown={(e) => e.stopPropagation()}
      style:left="{editing.x}px"
      style:top="{editing.y}px"
      style:color={color}
      style:font="600 {fontSize(size)}px {FONT}"
      rows={Math.max(1, editing.text.split("\n").length)}
    ></textarea>
  {/if}

  {#if sel && barPos && !drag}
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div class="bar" style:right="{barPos.right}px" style:top="{barPos.top}px" onmousedown={(e) => e.stopPropagation()}>
      <div class="row">
        {#each TOOLS as tl (tl.id)}
          <button class:on={tool === tl.id} title="{t(tl.name)} ({tl.key})" onclick={() => (tool = tool === tl.id ? null : tl.id)}>
            <tl.icon size={16} />
          </button>
        {/each}
        <span class="sep"></span>
        <button title="{t('撤销')} (⌘Z)" disabled={!shapes.length} onclick={undo}><Undo2 size={16} /></button>
        <span class="sep"></span>
        <button title="{t('钉在桌面上')} (F3)" onclick={() => finish("pin")}><Pin size={16} /></button>
        <button title="{t('保存')} (⌘S)" onclick={() => finish("save")}><Download size={16} /></button>
        <button title={t("退出")} onclick={cancel}><X size={16} /></button>
        <button class="ok" title="{t('复制')} (↵)" onclick={() => finish("copy")}><Check size={16} /></button>
      </div>
      {#if tool && tool !== "mosaic"}
        <div class="row sub">
          {#each SIZES as s (s)}
            <button class="dot-btn" class:on={size === s} title={t("粗细")} onclick={() => (size = s)}>
              <span class="dot" style:width="{s + 4}px" style:height="{s + 4}px"></span>
            </button>
          {/each}
          <span class="sep"></span>
          {#each COLORS as c (c)}
            <button class="swatch" class:on={color === c} style:background={c} aria-label={c} onclick={() => (color = c)}></button>
          {/each}
        </div>
      {:else if tool === "mosaic"}
        <div class="row sub">
          {#each SIZES as s (s)}
            <button class="dot-btn" class:on={size === s} title={t("格子大小")} onclick={() => (size = s)}>
              <Grid3x3 size={8 + s * 1.5} />
            </button>
          {/each}
        </div>
      {/if}
    </div>
  {/if}

  {#if showLens && src}
    <div class="lens" style:left="{lensPos.x}px" style:top="{lensPos.y}px">
      <canvas bind:this={lens} width={LENS * ZOOM} height={LENS * ZOOM}></canvas>
      <div class="lens-info">
        <div>{Math.round(mouse.x * src.scale)}, {Math.round(mouse.y * src.scale)}</div>
        <div class="hex"><span class="chip" style:background={hex}></span>{hex}</div>
        {#if !sel}<div class="hint">{t("C 复制颜色")}</div>{/if}
      </div>
    </div>
  {/if}
</div>

<style>
  :global(html),
  :global(body) {
    background: #000 !important;
  }
  .stage {
    position: fixed;
    inset: 0;
    overflow: hidden;
    user-select: none;
    -webkit-user-select: none;
  }
  .stage.pick {
    cursor: crosshair;
  }
  .stage.movable {
    cursor: move;
  }
  .stage.drawing {
    cursor: crosshair;
  }
  canvas {
    display: block;
  }
  .size {
    position: absolute;
    padding: 2px 7px;
    border-radius: 4px;
    background: rgba(0, 0, 0, 0.7);
    color: #fff;
    font: 12px var(--mono);
    pointer-events: none;
  }
  .handle {
    position: absolute;
    width: 9px;
    height: 9px;
    margin: -4.5px 0 0 -4.5px;
    border: 1.5px solid #2f8cff;
    border-radius: 50%;
    background: #fff;
  }
  .handle.nw,
  .handle.se {
    cursor: nwse-resize;
  }
  .handle.ne,
  .handle.sw {
    cursor: nesw-resize;
  }
  .handle.n,
  .handle.s {
    cursor: ns-resize;
  }
  .handle.e,
  .handle.w {
    cursor: ew-resize;
  }
  .bar {
    position: absolute;
    display: flex;
    flex-direction: column;
    align-items: flex-end;
    gap: 4px;
    cursor: default;
  }
  .row {
    display: flex;
    align-items: center;
    gap: 2px;
    padding: 4px;
    border-radius: 8px;
    background: rgba(32, 34, 40, 0.96);
    box-shadow: 0 6px 20px rgba(0, 0, 0, 0.45);
  }
  .row button {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 32px;
    height: 32px;
    padding: 0;
    border: 0;
    border-radius: 6px;
    background: transparent;
    color: rgba(255, 255, 255, 0.85);
  }
  .row button:hover:not(:disabled) {
    background: rgba(255, 255, 255, 0.1);
  }
  .row button.on {
    background: #2f8cff;
    color: #fff;
  }
  .row button.ok {
    color: #34c759;
  }
  .row button:disabled {
    opacity: 0.35;
  }
  .sep {
    width: 1px;
    height: 20px;
    margin: 0 4px;
    background: rgba(255, 255, 255, 0.15);
  }
  .sub button {
    width: 28px;
    height: 28px;
  }
  .dot {
    border-radius: 50%;
    background: currentColor;
  }
  .swatch {
    width: 20px !important;
    height: 20px !important;
    margin: 0 3px;
    border: 2px solid rgba(255, 255, 255, 0.25) !important;
    border-radius: 50% !important;
  }
  .swatch.on {
    border-color: #fff !important;
    box-shadow: 0 0 0 2px #2f8cff;
  }
  .text-input {
    position: absolute;
    min-width: 40px;
    height: auto;
    padding: 0;
    border: 1px dashed rgba(255, 255, 255, 0.7);
    border-radius: 0;
    background: transparent;
    line-height: 1.25;
    resize: none;
    overflow: hidden;
    white-space: pre;
    field-sizing: content;
    outline: none;
  }
  .lens {
    position: absolute;
    border: 1px solid rgba(255, 255, 255, 0.7);
    border-radius: 6px;
    overflow: hidden;
    background: rgba(20, 20, 24, 0.92);
    box-shadow: 0 6px 20px rgba(0, 0, 0, 0.5);
    pointer-events: none;
  }
  .lens canvas {
    display: block;
  }
  .lens-info {
    padding: 5px 8px;
    color: #fff;
    font: 11.5px var(--mono);
    line-height: 1.5;
  }
  .hex {
    display: flex;
    align-items: center;
    gap: 6px;
  }
  .chip {
    width: 10px;
    height: 10px;
    border: 1px solid rgba(255, 255, 255, 0.6);
    border-radius: 2px;
  }
  .hint {
    color: rgba(255, 255, 255, 0.55);
    font-family: var(--sans);
  }
</style>
