<script lang="ts">
  import { ClipboardList, House, Scissors, Search, Settings, Star } from "@lucide/svelte";
  import { api } from "../lib/api";
  import { app, openPage, openTool } from "../lib/store.svelte";
  import { t } from "../lib/i18n.svelte";
  import { isMac } from "../lib/util";
  import { TOOLS, findTool } from "../tools/registry";
  import { CATEGORIES } from "../tools/types";

  const favorites = $derived(app.favorites.map(findTool).filter((x) => x !== undefined));
</script>

<nav class="side" class:mac={isMac}>
  <div class="top" data-tauri-drag-region></div>
  <button class="search" onclick={() => (app.palette = true)}>
    <Search size={14} />
    <span>{t("搜索工具")}</span>
    <kbd>{isMac ? "⌘K" : "Ctrl+K"}</kbd>
  </button>

  <div class="scroll">
    <button class="item" class:on={app.page === "home"} onclick={() => openPage("home")}><House size={15} />{t("全部工具")}</button>
    <button class="item" class:on={app.page === "clipboard"} onclick={() => openPage("clipboard")}><ClipboardList size={15} />{t("剪贴板历史")}</button>
    <button class="item" onclick={() => api.shotStart(true)}><Scissors size={15} />{t("截图")}</button>

    {#if favorites.length}
      <div class="group"><Star size={11} />{t("收藏")}</div>
      {#each favorites as tool (tool.id)}
        <button class="item" class:on={app.page === "tool" && app.tool === tool.id} onclick={() => openTool(tool.id)}><tool.icon size={15} />{t(tool.name)}</button>
      {/each}
    {/if}

    {#each CATEGORIES as c (c.id)}
      <div class="group">{t(c.name)}</div>
      {#each TOOLS.filter((x) => x.category === c.id) as tool (tool.id)}
        <button class="item" class:on={app.page === "tool" && app.tool === tool.id} onclick={() => openTool(tool.id)}><tool.icon size={15} />{t(tool.name)}</button>
      {/each}
    {/each}
  </div>

  <button class="item settings" class:on={app.page === "settings"} onclick={() => openPage("settings")}><Settings size={15} />{t("设置")}</button>
</nav>

<style>
  .side {
    display: flex;
    flex-direction: column;
    width: 220px;
    height: 100%;
    background: var(--bg-side);
    border-right: 1px solid var(--border);
  }
  .top {
    flex: 0 0 12px;
  }
  .side.mac .top {
    /* 红绿灯按钮的位置 */
    flex-basis: 40px;
  }
  .search {
    display: flex;
    align-items: center;
    gap: 8px;
    margin: 0 10px 10px;
    height: 30px;
    color: var(--text-faint);
    background: var(--input-bg);
    text-align: left;
  }
  .search span {
    flex: 1;
  }
  .scroll {
    flex: 1;
    overflow-y: auto;
    padding: 0 8px 10px;
  }
  .group {
    display: flex;
    align-items: center;
    gap: 5px;
    margin: 14px 10px 4px;
    color: var(--text-faint);
    font-size: 11px;
    font-weight: 600;
  }
  .item {
    display: flex;
    align-items: center;
    gap: 9px;
    width: 100%;
    height: 30px;
    padding: 0 10px;
    border: 0;
    border-radius: 6px;
    background: transparent;
    color: var(--text-dim);
    text-align: left;
  }
  .item:hover {
    background: var(--bg-hover);
    color: var(--text);
  }
  .item.on {
    background: var(--bg-selected);
    color: var(--text);
  }
  .settings {
    width: auto;
    margin: 6px 8px 10px;
  }
</style>
