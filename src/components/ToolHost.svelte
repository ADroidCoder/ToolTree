<!-- 打开一个工具：标题栏（名字、说明、收藏）加工具本身，工具按需加载 -->
<script lang="ts">
  import { Star } from "@lucide/svelte";
  import type { Component } from "svelte";
  import { app, toggleFavorite } from "../lib/store.svelte";
  import { t } from "../lib/i18n.svelte";
  import { findTool } from "../tools/registry";
  import type { ToolProps } from "../tools/types";

  let { id }: { id: string } = $props();

  const tool = $derived(findTool(id));
  let Comp = $state<Component<ToolProps> | null>(null);

  $effect(() => {
    const tl = tool;
    Comp = null;
    if (!tl) return;
    let stale = false;
    tl.load().then((m) => !stale && (Comp = m.default));
    return () => (stale = true);
  });

  const input = $derived(app.handoff?.tool === id ? app.handoff.input : undefined);
  const key = $derived(`${id}:${app.handoff?.tool === id ? app.handoff.key : ""}`);
</script>

{#if tool}
  <div class="host">
    <header data-tauri-drag-region>
      <tool.icon size={18} />
      <h1>{t(tool.name)}</h1>
      <span class="desc">{t(tool.desc)}</span>
      <button class="ghost icon star" class:on={app.favorites.includes(id)} title={t("收藏")} onclick={() => toggleFavorite(id)}>
        <Star size={15} />
      </button>
    </header>
    <div class="body">
      {#if Comp}
        {#key key}
          <Comp {input} />
        {/key}
      {/if}
    </div>
  </div>
{/if}

<style>
  .host {
    display: flex;
    flex-direction: column;
    height: 100%;
  }
  header {
    display: flex;
    align-items: center;
    gap: 10px;
    flex: 0 0 52px;
    padding: 0 20px;
    border-bottom: 1px solid var(--border);
    color: var(--accent);
  }
  h1 {
    margin: 0;
    color: var(--text);
    font-size: 15px;
    font-weight: 600;
    white-space: nowrap;
  }
  .desc {
    color: var(--text-faint);
    font-size: 12px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .star {
    margin-left: auto;
    color: var(--text-faint);
  }
  .star.on {
    color: var(--warn);
  }
  .star.on :global(svg) {
    fill: currentColor;
  }
  .body {
    flex: 1;
    min-height: 0;
    padding: 16px 20px;
    overflow: auto;
  }
</style>
