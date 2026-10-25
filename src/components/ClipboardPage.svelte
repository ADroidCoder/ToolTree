<!-- 主窗口里的剪贴板历史：左边列表，右边完整内容和“用哪个工具打开” -->
<script lang="ts">
  import { onMount } from "svelte";
  import { listen } from "@tauri-apps/api/event";
  import { ask } from "@tauri-apps/plugin-dialog";
  import { Copy, Pin, PinOff, Trash2, ArrowRight } from "@lucide/svelte";
  import ClipList from "./ClipList.svelte";
  import ClipThumb from "./ClipThumb.svelte";
  import { api, type Clip, type Config } from "../lib/api";
  import { detect } from "../lib/detect";
  import { t } from "../lib/i18n.svelte";
  import { openPage, openTool, toast } from "../lib/store.svelte";
  import { findTool } from "../tools/registry";
  import { formatDateTime, formatSize } from "../lib/util";

  let selected = $state<Clip | null>(null);
  let full = $state<Clip | null>(null);
  let count = $state(0);
  let config = $state<Config | null>(null);
  let list = $state<ReturnType<typeof ClipList>>();

  // 列表里只有开头一部分文字，选中后取完整内容
  $effect(() => {
    const c = selected;
    full = c;
    if (!c || c.kind === "image") return;
    let stale = false;
    api.clipGet(c.id).then((x) => !stale && x && (full = x));
    return () => (stale = true);
  });

  const detections = $derived(full?.kind === "text" ? detect(full.text) : []);

  async function refresh() {
    count = await api.clipCount().catch(() => 0);
    config = await api.configGet().catch(() => null);
  }

  onMount(() => {
    refresh();
    list?.focus();
    const a = listen("clips-changed", refresh);
    const b = listen("config-changed", refresh);
    return () => {
      a.then((f) => f());
      b.then((f) => f());
    };
  });

  async function copyClip(c: Clip) {
    await api.clipCopy(c.id);
    toast(t("已复制"));
  }

  async function clearAll() {
    if (!(await ask(t("清空所有剪贴板历史？置顶的记录会保留。"), { title: t("清空历史"), kind: "warning" }))) return;
    const n = await api.clipClear();
    toast(t("已删除 {0} 条", n));
  }

  async function toggleRecording() {
    if (!config) return;
    await api.configSet({ ...config, clipEnabled: !config.clipEnabled });
    refresh();
  }
</script>

<div class="page">
  <div class="left">
    <ClipList bind:this={list} bind:selected onActivate={copyClip} />
    <div class="foot">
      <span>{t("{0} 条记录", count)}</span>
      {#if config}
        <label class="check"><input type="checkbox" checked={config.clipEnabled} onchange={toggleRecording} />{t("记录剪贴板")}</label>
      {/if}
      <button class="ghost link" onclick={() => openPage("settings")}>{t("设置")}</button>
      <button class="ghost link danger" onclick={clearAll}>{t("清空")}</button>
    </div>
  </div>
  <div class="detail">
    {#if full}
      <div class="actions">
        <button class="primary icon" onclick={() => full && copyClip(full)}><Copy size={13} />{t("复制")}</button>
        <button class="icon" onclick={() => full && api.clipPin(full.id, !full.pinned)}>
          {#if full.pinned}<PinOff size={13} />{t("取消置顶")}{:else}<Pin size={13} />{t("置顶")}{/if}
        </button>
        <button class="icon danger" onclick={() => full && api.clipRemove(full.id)}><Trash2 size={13} />{t("删除")}</button>
        <span class="info">
          {#if full.source}{full.source} · {/if}{formatDateTime(full.time)}
          · {full.kind === "image" ? `${full.width} × ${full.height} · ${formatSize(full.size)}` : full.kind === "files" ? t("{0} 个文件", full.size) : t("{0} 个字符", full.size)}
        </span>
      </div>

      {#if detections.length}
        <div class="detections">
          {#each detections as d (d.tool + d.label)}
            {@const tool = findTool(d.tool)}
            {#if tool}
              <button class="det" onclick={() => full && openTool(d.tool, full.text.trim())}>
                <tool.icon size={15} />
                <div class="det-body">
                  <div><b>{t(d.label)}</b> <span class="faint">→ {t(tool.name)}</span></div>
                  {#if d.preview}<div class="mono preview">{d.preview}</div>{/if}
                </div>
                <ArrowRight size={14} />
              </button>
            {/if}
          {/each}
        </div>
      {/if}

      <div class="content">
        {#if full.kind === "image"}
          <ClipThumb clip={full} full />
        {:else if full.kind === "files"}
          <div class="files mono">{#each full.text.split("\n") as p (p)}<div>{p}</div>{/each}</div>
        {:else}
          <pre class="mono">{full.text}</pre>
        {/if}
      </div>
    {:else}
      <div class="none faint">{t("选择一条记录查看完整内容")}</div>
    {/if}
  </div>
</div>

<style>
  .page {
    display: flex;
    height: 100%;
    min-height: 0;
  }
  .left {
    flex: 0 0 380px;
    display: flex;
    flex-direction: column;
    min-height: 0;
    border-right: 1px solid var(--border);
  }
  .left > :global(.clips) {
    flex: 1;
  }
  .foot {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 6px 12px;
    border-top: 1px solid var(--border);
    color: var(--text-faint);
    font-size: 12px;
  }
  .foot .check {
    margin-left: auto;
  }
  .link {
    height: 24px;
    padding: 0 6px;
    color: var(--text-dim);
    font-size: 12px;
  }
  .link.danger {
    color: var(--del);
  }
  .detail {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 12px;
    min-width: 0;
    padding: 14px 18px;
  }
  .actions {
    display: flex;
    align-items: center;
    gap: 8px;
  }
  .info {
    margin-left: auto;
    color: var(--text-faint);
    font-size: 12px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .detections {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  .det {
    display: flex;
    align-items: center;
    gap: 12px;
    height: auto;
    padding: 8px 12px;
    text-align: left;
    background: var(--bg-card);
  }
  .det:hover {
    border-color: var(--accent) !important;
  }
  .det-body {
    flex: 1;
    min-width: 0;
  }
  .preview {
    margin-top: 2px;
    color: var(--text-dim);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .content {
    flex: 1;
    min-height: 0;
    overflow: auto;
    border: 1px solid var(--border);
    border-radius: var(--radius);
    background: var(--bg-card);
  }
  pre,
  .files {
    margin: 0;
    padding: 12px 14px;
    white-space: pre-wrap;
    word-break: break-word;
    user-select: text;
    -webkit-user-select: text;
  }
  .none {
    margin: auto;
  }
</style>
