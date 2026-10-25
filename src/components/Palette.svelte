<!-- 命令面板：搜索工具；输入或粘贴的内容能识别时，直接推荐对应工具并带上这段内容 -->
<script lang="ts">
  import { ClipboardList, Settings, House } from "@lucide/svelte";
  import type { Component } from "svelte";
  import { app, openPage, openTool } from "../lib/store.svelte";
  import { detect } from "../lib/detect";
  import { t } from "../lib/i18n.svelte";
  import { findTool, searchTools } from "../tools/registry";

  interface Item {
    key: string;
    icon: Component;
    title: string;
    sub?: string;
    run: () => void;
  }

  let q = $state("");
  let index = $state(0);
  let input = $state<HTMLInputElement>();
  let listEl = $state<HTMLDivElement>();

  const PAGES: Item[] = [
    { key: "p:home", icon: House, title: "全部工具", run: () => openPage("home") },
    { key: "p:clipboard", icon: ClipboardList, title: "剪贴板历史", run: () => openPage("clipboard") },
    { key: "p:settings", icon: Settings, title: "设置", run: () => openPage("settings") },
  ];

  const items = $derived.by((): Item[] => {
    const out: Item[] = [];
    for (const d of detect(q)) {
      const tool = findTool(d.tool);
      if (tool) {
        const text = q.trim();
        out.push({ key: "d:" + d.tool + d.label, icon: tool.icon, title: `${t(d.label)} → ${t(tool.name)}`, sub: d.preview, run: () => openTool(tool.id, text) });
      }
    }
    for (const tool of searchTools(q)) out.push({ key: "t:" + tool.id, icon: tool.icon, title: t(tool.name), sub: t(tool.desc), run: () => openTool(tool.id) });
    const s = q.trim().toLowerCase();
    for (const p of PAGES) if (!s || t(p.title).toLowerCase().includes(s) || p.title.includes(s)) out.push({ ...p, title: t(p.title) });
    return out;
  });

  $effect(() => {
    void q;
    index = 0;
  });

  $effect(() => {
    input?.focus();
  });

  function close() {
    app.palette = false;
  }

  function run(it: Item | undefined) {
    if (!it) return;
    close();
    it.run();
  }

  function onKey(e: KeyboardEvent) {
    if (e.key === "ArrowDown") index = Math.min(items.length - 1, index + 1);
    else if (e.key === "ArrowUp") index = Math.max(0, index - 1);
    else if (e.key === "Enter") run(items[index]);
    else if (e.key === "Escape") close();
    else return;
    e.preventDefault();
    listEl?.querySelector(`[data-i="${index}"]`)?.scrollIntoView({ block: "nearest" });
  }
</script>

<!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
<div class="mask" onclick={close}>
  <div class="box" onclick={(e) => e.stopPropagation()}>
    <input bind:this={input} bind:value={q} onkeydown={onKey} placeholder={t("搜索工具，或粘贴内容自动识别")} spellcheck="false" />
    <div class="list" bind:this={listEl}>
      {#each items as it, i (it.key)}
        <button class="row" class:on={i === index} data-i={i} onmouseenter={() => (index = i)} onclick={() => run(it)}>
          <it.icon size={16} />
          <span class="title">{it.title}</span>
          {#if it.sub}<span class="sub">{it.sub}</span>{/if}
        </button>
      {:else}
        <div class="empty">{t("没有找到")}</div>
      {/each}
    </div>
  </div>
</div>

<style>
  .mask {
    position: fixed;
    inset: 0;
    z-index: 100;
    display: flex;
    justify-content: center;
    align-items: flex-start;
    padding-top: 12vh;
    background: rgba(0, 0, 0, 0.3);
  }
  .box {
    width: min(600px, 90vw);
    border: 1px solid var(--border);
    border-radius: var(--radius-lg);
    background: var(--bg-menu);
    box-shadow: var(--shadow);
    overflow: hidden;
  }
  input {
    width: 100%;
    height: 48px !important;
    padding: 0 16px !important;
    border: 0 !important;
    border-bottom: 1px solid var(--border) !important;
    border-radius: 0 !important;
    background: transparent !important;
    font-size: 15px !important;
  }
  .list {
    max-height: 50vh;
    overflow-y: auto;
    padding: 6px;
  }
  .row {
    display: flex;
    align-items: center;
    gap: 10px;
    width: 100%;
    height: 38px;
    padding: 0 10px;
    border: 0;
    border-radius: 6px;
    background: transparent;
    text-align: left;
    color: var(--text-dim);
  }
  .row.on {
    background: var(--bg-selected);
    color: var(--text);
  }
  .title {
    color: var(--text);
    white-space: nowrap;
  }
  .sub {
    flex: 1;
    min-width: 0;
    color: var(--text-faint);
    font-size: 12px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .empty {
    padding: 20px;
    color: var(--text-faint);
    text-align: center;
  }
</style>
