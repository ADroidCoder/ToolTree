<!-- 首页：粘贴任意内容自动识别，下面是全部工具 -->
<script lang="ts">
  import { ArrowRight, Sparkles } from "@lucide/svelte";
  import { app, openTool } from "../lib/store.svelte";
  import { detect } from "../lib/detect";
  import { t } from "../lib/i18n.svelte";
  import { TOOLS, findTool } from "../tools/registry";
  import { CATEGORIES } from "../tools/types";

  let text = $state("");
  const detections = $derived(detect(text));
  const recents = $derived(app.recents.map(findTool).filter((x) => x !== undefined).slice(0, 6));
</script>

<div class="home">
  <header data-tauri-drag-region></header>
  <div class="content">
    <section class="smart">
      <div class="title"><Sparkles size={15} />{t("智能识别")}</div>
      <textarea class="mono" rows="3" spellcheck="false" bind:value={text} placeholder={t("粘贴任意内容：JSON、时间戳、JWT、Base64、网址、颜色……自动推荐合适的工具")}></textarea>
      {#if text.trim()}
        <div class="dets">
          {#each detections as d (d.tool + d.label)}
            {@const tool = findTool(d.tool)}
            {#if tool}
              <button class="det" onclick={() => openTool(d.tool, text.trim())}>
                <tool.icon size={15} />
                <span><b>{t(d.label)}</b> → {t(tool.name)}</span>
                {#if d.preview}<span class="mono preview">{d.preview}</span>{/if}
                <ArrowRight size={14} />
              </button>
            {/if}
          {:else}
            <span class="faint">{t("没有识别出格式，可以在下面选择工具")}</span>
          {/each}
        </div>
      {/if}
    </section>

    {#if recents.length}
      <h2>{t("最近使用")}</h2>
      <div class="grid">
        {#each recents as tool (tool.id)}
          <button class="card" onclick={() => openTool(tool.id)}>
            <span class="icon"><tool.icon size={18} /></span>
            <span class="name">{t(tool.name)}</span>
            <span class="desc">{t(tool.desc)}</span>
          </button>
        {/each}
      </div>
    {/if}

    {#each CATEGORIES as c (c.id)}
      <h2>{t(c.name)}</h2>
      <div class="grid">
        {#each TOOLS.filter((x) => x.category === c.id) as tool (tool.id)}
          <button class="card" onclick={() => openTool(tool.id)}>
            <span class="icon"><tool.icon size={18} /></span>
            <span class="name">{t(tool.name)}</span>
            <span class="desc">{t(tool.desc)}</span>
          </button>
        {/each}
      </div>
    {/each}
  </div>
</div>

<style>
  .home {
    display: flex;
    flex-direction: column;
    height: 100%;
  }
  header {
    flex: 0 0 24px;
  }
  .content {
    flex: 1;
    overflow-y: auto;
    padding: 0 28px 28px;
  }
  .smart {
    padding: 14px 16px;
    border: 1px solid var(--border);
    border-radius: var(--radius-lg);
    background: var(--bg-card);
  }
  .title {
    display: flex;
    align-items: center;
    gap: 6px;
    margin-bottom: 10px;
    color: var(--accent);
    font-weight: 600;
  }
  .smart textarea {
    width: 100%;
    resize: vertical;
  }
  .dets {
    display: flex;
    flex-direction: column;
    gap: 6px;
    margin-top: 10px;
  }
  .det {
    display: flex;
    align-items: center;
    gap: 10px;
    height: 34px;
    text-align: left;
  }
  .det:hover {
    border-color: var(--accent) !important;
  }
  .preview {
    flex: 1;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    color: var(--text-dim);
  }
  h2 {
    margin: 24px 0 10px;
    color: var(--text-dim);
    font-size: 13px;
    font-weight: 600;
  }
  .grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(220px, 1fr));
    gap: 10px;
  }
  .card {
    display: grid;
    grid-template-columns: auto 1fr;
    grid-template-rows: auto auto;
    column-gap: 10px;
    row-gap: 3px;
    height: auto;
    padding: 12px 14px;
    border-radius: var(--radius);
    background: var(--bg-card);
    text-align: left;
  }
  .card:hover {
    background: var(--bg-card-hover);
    border-color: var(--border) !important;
  }
  .icon {
    grid-row: span 2;
    display: flex;
    align-items: center;
    justify-content: center;
    width: 34px;
    height: 34px;
    border-radius: 8px;
    background: var(--bg-hover);
    color: var(--accent);
  }
  .name {
    font-weight: 600;
  }
  .desc {
    color: var(--text-faint);
    font-size: 12px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
</style>
