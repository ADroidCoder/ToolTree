<!-- 钉在桌面上的截图：拖动移动，滚轮缩放，双击或 Esc 关闭，右键菜单复制、保存 -->
<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { getCurrentWindow, LogicalSize } from "@tauri-apps/api/window";
  import { t, watchLang } from "../lib/i18n.svelte";
  import { applyTheme, watchSystemTheme } from "../lib/settings.svelte";
  import { decodeRaw } from "../lib/shot/draw";

  const win = getCurrentWindow();
  let canvas = $state<HTMLCanvasElement>();
  let bitmap: ImageBitmap | null = null;
  /** 原始大小（CSS 像素）和当前缩放 */
  let baseW = 0;
  let baseH = 0;
  let zoom = $state(1);
  let showZoom = $state(false);
  let menu = $state<{ x: number; y: number } | null>(null);
  let zoomTimer: ReturnType<typeof setTimeout> | undefined;

  function draw() {
    if (!canvas || !bitmap) return;
    const k = devicePixelRatio || 1;
    canvas.width = Math.round(innerWidth * k);
    canvas.height = Math.round(innerHeight * k);
    const ctx = canvas.getContext("2d")!;
    ctx.imageSmoothingQuality = "high";
    ctx.drawImage(bitmap, 0, 0, canvas.width, canvas.height);
  }

  onMount(() => {
    applyTheme();
    const stopTheme = watchSystemTheme();
    const stopLang = watchLang();
    (async () => {
      const img = await decodeRaw(await invoke<ArrayBuffer>("pin_image"));
      bitmap = img.bitmap;
      baseW = innerWidth;
      baseH = innerHeight;
      draw();
      await win.show();
    })();
    addEventListener("resize", draw);
    return () => {
      stopTheme();
      stopLang();
      removeEventListener("resize", draw);
    };
  });

  function setZoom(z: number) {
    zoom = Math.max(0.1, Math.min(8, z));
    win.setSize(new LogicalSize(Math.max(20, baseW * zoom), Math.max(20, baseH * zoom)));
    showZoom = true;
    clearTimeout(zoomTimer);
    zoomTimer = setTimeout(() => (showZoom = false), 800);
  }

  function onWheel(e: WheelEvent) {
    e.preventDefault();
    setZoom(zoom * Math.pow(1.0015, -e.deltaY));
  }

  function action(a: "copy" | "save" | "close") {
    menu = null;
    invoke("pin_action", { action: a });
  }

  function onDown(e: MouseEvent) {
    if (e.button !== 0) return;
    menu = null;
    win.startDragging();
  }

  function onKey(e: KeyboardEvent) {
    if (e.key === "Escape") action("close");
    else if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === "c") action("copy");
    else if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === "s") action("save");
    else if (e.key === "0") setZoom(1);
    else return;
    e.preventDefault();
  }
</script>

<svelte:window onkeydown={onKey} />

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
  class="pin"
  onmousedown={onDown}
  ondblclick={() => action("close")}
  onwheel={onWheel}
  oncontextmenu={(e) => {
    e.preventDefault();
    menu = { x: Math.min(e.clientX, innerWidth - 150), y: Math.min(e.clientY, innerHeight - 140) };
  }}
>
  <canvas bind:this={canvas}></canvas>
  {#if showZoom}<div class="zoom">{Math.round(zoom * 100)}%</div>{/if}
</div>

{#if menu}
  <div class="menu" style:left="{Math.max(0, menu.x)}px" style:top="{Math.max(0, menu.y)}px">
    <button onclick={() => action("copy")}>{t("复制")}</button>
    <button onclick={() => action("save")}>{t("保存")}…</button>
    <button onclick={() => ((menu = null), setZoom(1))}>{t("原始大小")}</button>
    <button onclick={() => action("close")}>{t("关闭")}</button>
  </div>
{/if}

<style>
  .pin {
    position: fixed;
    inset: 0;
    cursor: grab;
    outline: 1px solid rgba(47, 140, 255, 0.6);
    outline-offset: -1px;
  }
  canvas {
    display: block;
    width: 100%;
    height: 100%;
  }
  .zoom {
    position: absolute;
    left: 50%;
    top: 50%;
    transform: translate(-50%, -50%);
    padding: 4px 10px;
    border-radius: 6px;
    background: rgba(0, 0, 0, 0.7);
    color: #fff;
    font: 13px var(--mono);
    pointer-events: none;
  }
  .menu {
    position: fixed;
    z-index: 10;
    display: flex;
    flex-direction: column;
    min-width: 130px;
    padding: 4px;
    border: 1px solid var(--border);
    border-radius: 7px;
    background: var(--bg-menu);
    box-shadow: var(--shadow);
  }
  .menu button {
    height: 28px;
    border: 0;
    background: transparent;
    text-align: left;
  }
  .menu button:hover {
    background: var(--bg-selected);
  }
</style>
