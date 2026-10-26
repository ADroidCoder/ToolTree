<script lang="ts">
  import { onMount } from "svelte";
  import { listen } from "@tauri-apps/api/event";
  import { enable, disable, isEnabled } from "@tauri-apps/plugin-autostart";
  import ShortcutInput from "./ShortcutInput.svelte";
  import Seg from "./Seg.svelte";
  import { api, type Config } from "../lib/api";
  import { i18n, setLang, t, type Lang } from "../lib/i18n.svelte";
  import { setTheme, settings, type Theme } from "../lib/settings.svelte";
  import { toast } from "../lib/store.svelte";
  import { isMac } from "../lib/util";
  import { checkUpdate, updater } from "../lib/updater.svelte";

  let config = $state<Config | null>(null);
  let autostart = $state(false);
  let trusted = $state(true);
  let ignoreText = $state("");

  async function load() {
    config = await api.configGet();
    ignoreText = config.ignoreApps.join("\n");
    trusted = await api.accessibilityTrusted();
  }

  onMount(() => {
    load();
    isEnabled().then((v) => (autostart = v)).catch(() => {});
    // 从系统设置授权回来后刷新状态
    const onFocus = () => api.accessibilityTrusted().then((v) => (trusted = v));
    addEventListener("focus", onFocus);
    const un = listen("config-changed", load);
    return () => {
      removeEventListener("focus", onFocus);
      un.then((f) => f());
    };
  });

  async function save(patch: Partial<Config>) {
    if (!config) return;
    const next = { ...config, ...patch };
    config = next;
    try {
      await api.configSet(next);
    } catch (e) {
      toast(t("快捷键注册失败，可能被其他应用占用了：\n{0}", String(e)), "error");
    }
  }

  async function setAutostart(on: boolean) {
    try {
      await (on ? enable() : disable());
      autostart = on;
    } catch (e) {
      toast(String(e), "error");
    }
  }

  let lang = $state<Lang | "system">(i18n.choice ?? "system");
  $effect(() => setLang(lang === "system" ? null : lang));
  let theme = $state<Theme>(settings.theme);
  $effect(() => setTheme(theme));
</script>

<div class="page">
  <header data-tauri-drag-region><h1>{t("设置")}</h1></header>
  <div class="content">
    <section>
      <h2>{t("外观")}</h2>
      <div class="row">
        <span>{t("主题")}</span>
        <Seg bind:value={theme} options={[{ value: "system", label: t("跟随系统") }, { value: "light", label: t("浅色") }, { value: "dark", label: t("深色") }]} />
      </div>
      <div class="row">
        <span>{t("语言")}</span>
        <Seg bind:value={lang} options={[{ value: "system", label: t("跟随系统") }, { value: "zh", label: "中文" }, { value: "en", label: "English" }]} />
      </div>
    </section>

    {#if config}
      <section>
        <h2>{t("全局快捷键")}</h2>
        <div class="row">
          <span>{t("搜索工具")}<small>{t("在任何地方打开 ToolTree 并搜索工具")}</small></span>
          <ShortcutInput value={config.launcherShortcut} onchange={(v) => save({ launcherShortcut: v })} />
        </div>
        <div class="row">
          <span>{t("剪贴板历史")}<small>{t("在鼠标位置弹出剪贴板面板")}</small></span>
          <ShortcutInput value={config.clipShortcut} onchange={(v) => save({ clipShortcut: v })} />
        </div>
        <div class="row">
          <span>{t("截图")}<small>{t("框选区域、标注，复制、保存或钉在桌面上")}</small></span>
          <ShortcutInput value={config.shotShortcut} onchange={(v) => save({ shotShortcut: v })} />
        </div>
      </section>

      <section>
        <h2>{t("剪贴板")}</h2>
        <div class="row">
          <span>{t("记录剪贴板历史")}</span>
          <input type="checkbox" checked={config.clipEnabled} onchange={(e) => save({ clipEnabled: e.currentTarget.checked })} />
        </div>
        <div class="row">
          <span>{t("最多保留")}<small>{t("置顶的记录不算在内")}</small></span>
          <select value={config.clipMax} onchange={(e) => save({ clipMax: Number(e.currentTarget.value) })}>
            {#each [100, 500, 1000, 5000, 10000] as n (n)}<option value={n}>{t("{0} 条", n)}</option>{/each}
          </select>
        </div>
        <div class="row">
          <span>{t("保留时间")}</span>
          <select value={config.clipDays} onchange={(e) => save({ clipDays: Number(e.currentTarget.value) })}>
            {#each [[1, "1 天"], [7, "7 天"], [30, "30 天"], [90, "90 天"], [0, "永久"]] as const as [n, name] (n)}<option value={n}>{t(name)}</option>{/each}
          </select>
        </div>
        <div class="row">
          <span>{t("选中后直接粘贴")}<small>{t("在面板里按回车，自动粘贴到之前的应用；关闭后只复制")}</small></span>
          <input type="checkbox" checked={config.autoPaste} onchange={(e) => save({ autoPaste: e.currentTarget.checked })} />
        </div>
        {#if isMac && config.autoPaste && !trusted}
          <div class="warn">
            {t("自动粘贴需要在“系统设置 → 隐私与安全性 → 辅助功能”里允许 ToolTree。")}
            <button onclick={() => api.openAccessibilitySettings()}>{t("打开系统设置")}</button>
          </div>
        {/if}
        {#if isMac}
          <div class="row top">
            <span>{t("忽略这些应用")}<small>{t("每行一个应用名，在这些应用里复制的内容不记录。密码管理器标记为隐藏的内容本来就不会记录")}</small></span>
            <textarea rows="4" bind:value={ignoreText} onblur={() => save({ ignoreApps: ignoreText.split("\n").map((s) => s.trim()).filter(Boolean) })}></textarea>
          </div>
        {/if}
      </section>
    {/if}

    <section>
      <h2>{t("通用")}</h2>
      <div class="row">
        <span>{t("开机时启动")}<small>{t("启动后只在{0}里待命，不打开窗口", isMac ? t("菜单栏") : t("托盘"))}</small></span>
        <input type="checkbox" checked={autostart} onchange={(e) => setAutostart(e.currentTarget.checked)} />
      </div>
      <div class="row">
        <span>{t("版本")}<small>ToolTree {updater.version}{updater.update ? " · " + t("新版本 {0}", updater.update.version) : ""}</small></span>
        {#if updater.update}
          <button class="primary" onclick={() => (updater.open = true)}>{t("查看更新")}</button>
        {:else}
          <button onclick={() => checkUpdate(true)} disabled={updater.status === "checking"}>
            {updater.status === "checking" ? t("正在检查…") : t("检查更新")}
          </button>
        {/if}
      </div>
    </section>
  </div>
</div>

<style>
  .page {
    display: flex;
    flex-direction: column;
    height: 100%;
  }
  header {
    flex: 0 0 52px;
    display: flex;
    align-items: center;
    padding: 0 28px;
    border-bottom: 1px solid var(--border);
  }
  h1 {
    margin: 0;
    font-size: 15px;
  }
  .content {
    flex: 1;
    overflow-y: auto;
    padding: 8px 28px 28px;
  }
  section {
    max-width: 720px;
  }
  h2 {
    margin: 22px 0 8px;
    color: var(--text-dim);
    font-size: 12px;
    font-weight: 600;
  }
  .row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 20px;
    min-height: 46px;
    padding: 8px 14px;
    border: 1px solid var(--border);
    border-bottom: 0;
    background: var(--bg-card);
  }
  .row.top {
    align-items: flex-start;
  }
  section .row:first-of-type {
    border-radius: var(--radius) var(--radius) 0 0;
  }
  section .row:last-child {
    border-bottom: 1px solid var(--border);
    border-radius: 0 0 var(--radius) var(--radius);
  }
  .row > span:first-child {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  small {
    color: var(--text-faint);
    font-size: 11.5px;
  }
  .row textarea {
    width: 240px;
  }
  .warn {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 8px 14px;
    border: 1px solid var(--border);
    border-bottom: 0;
    background: var(--bg-hit);
    font-size: 12px;
  }
  .warn button {
    margin-left: auto;
    flex-shrink: 0;
  }
</style>
