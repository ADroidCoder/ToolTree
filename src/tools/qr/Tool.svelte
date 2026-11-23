<script lang="ts">
  import QRCode from "qrcode";
  import { untrack } from "svelte";
  import { save } from "@tauri-apps/plugin-dialog";
  import { Copy, Download } from "@lucide/svelte";
  import Pane from "../../components/Pane.svelte";
  import Seg from "../../components/Seg.svelte";
  import { api } from "../../lib/api";
  import { t } from "../../lib/i18n.svelte";
  import { persisted } from "../../lib/persist.svelte";
  import { toast } from "../../lib/store.svelte";
  import type { ToolProps } from "../types";
  import { qrOptions, qrVersion, type QrConfig, type QrLevel } from "./qr";

  let { input }: ToolProps = $props();

  const text = persisted("qr.input", "https://github.com/ADroidCoder/ToolTree");
  const level = persisted<QrLevel>("qr.level", "M");
  const size = persisted("qr.size", 512);
  const margin = persisted("qr.margin", 4);
  const dark = persisted("qr.dark", "#000000");
  const light = persisted("qr.light", "#ffffff");
  const init = untrack(() => input);
  if (init !== undefined) text.value = init;

  let svg = $state("");
  let version = $state(0);
  let error = $state("");
  let busy = $state(false);

  const config = $derived<QrConfig>({ level: level.value, size: size.value, margin: margin.value, dark: dark.value, light: light.value });
  const ready = $derived(!!text.value && !!svg && !error);

  $effect(() => {
    const value = text.value;
    const options = { ...qrOptions(config), type: "svg" as const };
    const correction = config.level;
    let stale = false;
    svg = "";
    version = 0;
    error = "";
    if (!value) return;
    QRCode.toString(value, options)
      .then((out) => {
        if (stale) return;
        svg = out;
        version = qrVersion(value, correction);
      })
      .catch((e) => {
        if (stale) return;
        error = /too big|overflow/i.test(String(e))
          ? t("内容太长，无法生成二维码，请缩短内容或降低容错级别")
          : t("生成失败：{0}", String(e));
      });
    return () => (stale = true);
  });

  async function png() {
    return QRCode.toDataURL(text.value, { ...qrOptions(config), type: "image/png" });
  }

  async function copyImage() {
    if (!ready) return;
    busy = true;
    try {
      const data = await png();
      await api.qrCopy(data.slice(data.indexOf(",") + 1));
      toast(t("已复制"));
    } catch (e) {
      toast(t("复制失败：{0}", String(e)), "error");
    } finally {
      busy = false;
    }
  }

  async function exportQr(kind: "png" | "svg") {
    if (!ready) return;
    busy = true;
    try {
      const path = await save({
        defaultPath: `qrcode.${kind}`,
        filters: [{ name: kind.toUpperCase(), extensions: [kind] }],
      });
      if (!path) return;
      const data = kind === "png" ? await png() : svg;
      await api.qrSave(path, kind, kind === "png" ? data.slice(data.indexOf(",") + 1) : data);
      toast(t("已保存"));
    } catch (e) {
      toast(t("保存失败：{0}", String(e)), "error");
    } finally {
      busy = false;
    }
  }
</script>

<div class="tool">
  <div class="tool-bar">
    <span class="dim">{t("容错级别")}</span>
    <Seg
      bind:value={level.value}
      options={[
        { value: "L", label: "L · 7%" },
        { value: "M", label: "M · 15%" },
        { value: "Q", label: "Q · 25%" },
        { value: "H", label: "H · 30%" },
      ]}
    />
    <label class="field">{t("尺寸")}<input type="number" min="128" max="2048" step="128" bind:value={size.value} /><span>px</span></label>
    <label class="field">{t("边距")}<input type="number" min="0" max="10" bind:value={margin.value} /></label>
  </div>
  <div class="tool-bar">
    <label class="color"><input type="color" bind:value={dark.value} />{t("前景色")}</label>
    <label class="color"><input type="color" bind:value={light.value} />{t("背景色")}</label>
    <span class="faint">{t("颜色对比不足时可能无法识别")}</span>
  </div>

  <div class="tool-cols">
    <Pane label={t("文本或网址")}>
      {#snippet actions()}
        <button onclick={() => (text.value = "")} disabled={!text.value}>{t("清空")}</button>
      {/snippet}
      <textarea class="code wrap" spellcheck="false" bind:value={text.value} placeholder={t("输入文本、网址或其他内容")}></textarea>
    </Pane>

    <div class="preview-pane">
      <Pane label={t("二维码预览")} grow={false}>
        {#snippet actions()}
          <button class="icon" onclick={copyImage} disabled={!ready || busy} title={t("复制图片")}><Copy size={13} />{t("复制图片")}</button>
          <button class="icon" onclick={() => exportQr("png")} disabled={!ready || busy}><Download size={13} />PNG</button>
          <button class="icon" onclick={() => exportQr("svg")} disabled={!ready || busy}><Download size={13} />SVG</button>
        {/snippet}
        <div class="preview">
          {#if svg}
            <div class="qr">{@html svg}</div>
          {:else if !error}
            <span class="faint">{t("输入文本或网址后自动生成")}</span>
          {/if}
        </div>
        <div class="meta faint">
          {#if version}{t("版本 {0}", version)} · {qrOptions(config).width} × {qrOptions(config).width} px{/if}
        </div>
      </Pane>
    </div>
  </div>
  {#if error}<div class="tool-error">{error}</div>{/if}
</div>

<style>
  .field,
  .color {
    display: inline-flex;
    align-items: center;
    gap: 6px;
  }
  .field input {
    width: 76px;
  }
  .color input {
    width: 34px;
    height: 28px;
    padding: 2px;
    border: 1px solid var(--border);
    border-radius: 5px;
    background: var(--input-bg);
    cursor: pointer;
  }
  .wrap {
    white-space: pre-wrap;
    overflow-wrap: anywhere;
  }
  .preview-pane {
    flex: 0 0 360px;
    min-width: 0;
  }
  .preview-pane :global(.pane),
  .preview-pane :global(.body) {
    height: 100%;
  }
  .preview {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    min-height: 280px;
    padding: 16px;
    overflow: hidden;
    border: 1px solid var(--border);
    border-radius: var(--radius);
    background: repeating-conic-gradient(var(--bg-empty) 0 25%, transparent 0 50%) 0 0 / 16px 16px;
  }
  .qr {
    width: 100%;
    max-width: 320px;
    line-height: 0;
  }
  .qr :global(svg) {
    display: block;
    width: 100%;
    height: auto;
  }
  .meta {
    min-height: 18px;
    padding-top: 6px;
    text-align: center;
    font-size: 12px;
  }
  @media (max-width: 900px) {
    .preview-pane {
      flex-basis: 300px;
    }
  }
</style>
