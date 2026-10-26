<script lang="ts">
  import { onMount } from "svelte";
  import { listen } from "@tauri-apps/api/event";
  import Sidebar from "./components/Sidebar.svelte";
  import Home from "./components/Home.svelte";
  import ToolHost from "./components/ToolHost.svelte";
  import Palette from "./components/Palette.svelte";
  import ClipboardPage from "./components/ClipboardPage.svelte";
  import SettingsPage from "./components/SettingsPage.svelte";
  import UpdateDialog from "./components/UpdateDialog.svelte";
  import { startUpdater, updater } from "./lib/updater.svelte";
  import { app, openPage, openTool } from "./lib/store.svelte";
  import { applyTheme, watchSystemTheme } from "./lib/settings.svelte";
  import { syncBackendLang, watchLang } from "./lib/i18n.svelte";
  import { modKey } from "./lib/util";
  import { api } from "./lib/api";
  import { t } from "./lib/i18n.svelte";
  import { toast } from "./lib/store.svelte";

  onMount(() => {
    applyTheme();
    syncBackendLang();
    const stopTheme = watchSystemTheme();
    const stopLang = watchLang();
    startUpdater();
    // 全局快捷键、托盘菜单、剪贴板面板发来的
    const a = listen("open-palette", () => (app.palette = true));
    const b = listen<{ tool: string | null; input: string | null }>("open-tool", (e) => {
      const { tool, input } = e.payload;
      if (tool === "clipboard") openPage("clipboard");
      else if (tool) openTool(tool, input ?? undefined);
    });
    const c = listen<string>("toast", (e) => toast(e.payload, "error"));
    const d = listen("shot-permission", () => (needScreenPermission = true));
    return () => {
      stopTheme();
      stopLang();
      for (const u of [a, b, c, d]) u.then((f) => f());
    };
  });

  /** 截图时发现没有屏幕录制权限 */
  let needScreenPermission = $state(false);

  function onGlobalKey(e: KeyboardEvent) {
    const key = e.key.toLowerCase();
    // 重新加载页面没有意义，还会丢掉没保存的输入
    if (e.key === "F5" || (modKey(e) && key === "r")) e.preventDefault();
    if (!modKey(e)) return;
    if (key === "k" || key === "p") app.palette = !app.palette;
    else if (key === ",") openPage("settings");
    else if (key === "1") openPage("home");
    else if (key === "2") openPage("clipboard");
    else return;
    e.preventDefault();
  }
</script>

<svelte:window onkeydown={onGlobalKey} />

<div class="app">
  <Sidebar />
  <main>
    {#if app.page === "home"}
      <Home />
    {:else if app.page === "clipboard"}
      <ClipboardPage />
    {:else if app.page === "settings"}
      <SettingsPage />
    {:else}
      <ToolHost id={app.tool} />
    {/if}
  </main>
</div>

{#if needScreenPermission}
  <div class="notice">
    <b>{t("截图需要“屏幕录制”权限")}</b>
    <p>{t("在“系统设置 → 隐私与安全性 → 屏幕录制”里打开 ToolTree，然后重新启动 ToolTree。")}</p>
    <div class="notice-actions">
      <button onclick={() => (needScreenPermission = false)}>{t("稍后")}</button>
      <button class="primary" onclick={() => api.openScreenRecordingSettings()}>{t("打开系统设置")}</button>
    </div>
  </div>
{/if}

{#if app.palette}
  <Palette />
{/if}

{#if updater.open && updater.update}
  <UpdateDialog />
{/if}

<div class="toasts">
  {#each app.toasts as t (t.id)}
    <div class="toast {t.kind}">{t.text}</div>
  {/each}
</div>

<style>
  .app {
    display: flex;
    height: 100vh;
  }
  main {
    flex: 1;
    min-width: 0;
    background: var(--bg-graph);
  }
  .notice {
    position: fixed;
    right: 16px;
    top: 16px;
    z-index: 150;
    width: 360px;
    padding: 14px 16px;
    border: 1px solid var(--border);
    border-radius: var(--radius-lg);
    background: var(--bg-menu);
    box-shadow: var(--shadow);
  }
  .notice p {
    margin: 6px 0 12px;
    color: var(--text-dim);
    line-height: 1.5;
  }
  .notice-actions {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
  }
  .toasts {
    position: fixed;
    right: 16px;
    bottom: 16px;
    z-index: 200;
    display: flex;
    flex-direction: column;
    gap: 8px;
    max-width: 460px;
  }
  .toast {
    padding: 9px 14px;
    background: var(--bg-menu);
    border: 1px solid var(--border);
    border-left: 4px solid var(--accent);
    border-radius: 6px;
    box-shadow: var(--shadow);
    white-space: pre-wrap;
    word-break: break-word;
  }
  .toast.error {
    border-left-color: var(--del);
  }
</style>
