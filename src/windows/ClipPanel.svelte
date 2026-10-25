<!-- 全局快捷键弹出的剪贴板面板：回车粘贴到之前的应用，Esc 关闭 -->
<script lang="ts">
  import { onMount } from "svelte";
  import { listen } from "@tauri-apps/api/event";
  import { Maximize2 } from "@lucide/svelte";
  import ClipList from "../components/ClipList.svelte";
  import { api, type Clip, type Config } from "../lib/api";
  import { detect } from "../lib/detect";
  import { t, watchLang } from "../lib/i18n.svelte";
  import { applyTheme, watchSystemTheme } from "../lib/settings.svelte";
  import { isMac, modKey } from "../lib/util";

  let list = $state<ReturnType<typeof ClipList>>();
  let selected = $state<Clip | null>(null);
  let config = $state<Config | null>(null);
  let trusted = $state(true);

  async function onShow() {
    list?.reset();
    list?.focus();
    config = await api.configGet().catch(() => null);
    trusted = await api.accessibilityTrusted().catch(() => true);
  }

  onMount(() => {
    applyTheme();
    const stopTheme = watchSystemTheme();
    const stopLang = watchLang();
    const un = listen("panel-show", onShow);
    onShow();
    return () => {
      stopTheme();
      stopLang();
      un.then((f) => f());
    };
  });

  async function paste(c: Clip) {
    await api.clipPaste(c.id).catch(() => {});
  }

  /** 选中的文本能识别成某种格式时，⌘O 在对应工具里打开 */
  async function openInTool() {
    if (!selected || selected.kind !== "text") return;
    const d = detect(selected.text)[0];
    const full = await api.clipGet(selected.id);
    api.showMain(d?.tool ?? "case", full?.text.trim() ?? selected.text);
  }

  function onKey(e: KeyboardEvent) {
    if (e.key === "Escape") api.panelHide();
    else if (modKey(e) && e.key.toLowerCase() === "o") openInTool();
    else if (modKey(e) && e.key.toLowerCase() === "c" && selected && !getSelection()?.toString()) api.clipCopy(selected.id).then(() => api.panelHide());
    else return;
    e.preventDefault();
  }

  const mod = isMac ? "⌘" : "Ctrl+";
</script>

<svelte:window onkeydown={onKey} />

<div class="panel">
  {#if config?.autoPaste && !trusted}
    <div class="banner">
      {t("自动粘贴需要“辅助功能”权限，现在只会复制。")}
      <button class="ghost" onclick={() => api.openAccessibilitySettings()}>{t("去授权")}</button>
    </div>
  {/if}
  <div class="body">
    <ClipList bind:this={list} bind:selected compact onActivate={paste} />
  </div>
  <div class="foot">
    <span><kbd>↵</kbd> {config?.autoPaste && trusted ? t("粘贴") : t("复制")}</span>
    <span><kbd>{mod}O</kbd> {t("用工具打开")}</span>
    <span><kbd>{mod}P</kbd> {t("置顶")}</span>
    <button class="ghost icon" title={t("在主窗口中打开")} onclick={() => api.showMain("clipboard").then(() => api.panelHide())}><Maximize2 size={13} /></button>
  </div>
</div>

<style>
  .panel {
    display: flex;
    flex-direction: column;
    height: 100vh;
    background: var(--bg-graph);
    border: 1px solid var(--border);
    border-radius: 10px;
    overflow: hidden;
  }
  .banner {
    display: flex;
    align-items: center;
    gap: 8px;
    margin: 10px 10px 0;
    padding: 6px 10px;
    border-radius: 6px;
    background: var(--bg-hit);
    font-size: 12px;
  }
  .banner button {
    margin-left: auto;
    height: 22px;
    color: var(--accent);
    font-size: 12px;
  }
  .body {
    flex: 1;
    min-height: 0;
  }
  .foot {
    display: flex;
    align-items: center;
    gap: 14px;
    padding: 6px 12px;
    border-top: 1px solid var(--border);
    background: var(--bg-side);
    color: var(--text-faint);
    font-size: 11px;
  }
  .foot button {
    margin-left: auto;
    height: 22px;
    padding: 0 6px;
    color: var(--text-dim);
  }
</style>
