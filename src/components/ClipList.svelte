<!-- 剪贴板历史列表：搜索、按类型筛选、键盘上下选择。
     主窗口的剪贴板页面和快捷键弹出的面板共用，选中后做什么由 onActivate 决定 -->
<script lang="ts">
  import { onMount } from "svelte";
  import { listen } from "@tauri-apps/api/event";
  import { File, Image, Pin, Search, Type } from "@lucide/svelte";
  import { api, type Clip, type ClipKind } from "../lib/api";
  import { detect } from "../lib/detect";
  import { t } from "../lib/i18n.svelte";
  import { modKey, relativeTime } from "../lib/util";
  import ClipThumb from "./ClipThumb.svelte";

  let {
    compact = false,
    selected = $bindable<Clip | null>(null),
    onActivate,
  }: {
    /** 面板里用：更紧凑，数字键快速选择 */
    compact?: boolean;
    selected?: Clip | null;
    onActivate: (c: Clip) => void;
  } = $props();

  type Filter = "all" | ClipKind | "pinned";
  const PAGE = 100;

  let search = $state("");
  let filter = $state<Filter>("all");
  let items = $state<Clip[]>([]);
  let more = $state(false);
  let index = $state(0);
  let input = $state<HTMLInputElement>();
  let listEl = $state<HTMLDivElement>();

  const FILTERS: { id: Filter; name: string }[] = [
    { id: "all", name: "全部" },
    { id: "text", name: "文本" },
    { id: "image", name: "图片" },
    { id: "files", name: "文件" },
    { id: "pinned", name: "置顶" },
  ];

  function query(offset: number) {
    return api.clipList({
      search: search.trim(),
      kind: filter === "all" || filter === "pinned" ? null : filter,
      pinned: filter === "pinned",
      offset,
      limit: PAGE,
    });
  }

  let seq = 0;
  export async function reload() {
    const my = ++seq;
    const list = await query(0).catch(() => []);
    if (my !== seq) return;
    const keep = selected?.id;
    items = list;
    more = list.length === PAGE;
    const i = items.findIndex((c) => c.id === keep);
    // 选中的那条还在就继续选中它，被删了就选同一位置的下一条
    index = Math.max(0, i >= 0 ? i : Math.min(index, items.length - 1));
    selected = items[index] ?? null;
  }

  async function loadMore() {
    if (!more) return;
    const my = seq;
    const list = await query(items.length).catch(() => []);
    if (my !== seq) return;
    items = [...items, ...list];
    more = list.length === PAGE;
  }

  export function focus() {
    input?.focus();
    input?.select();
  }

  /** 面板每次打开时回到最上面那条 */
  export function reset() {
    search = "";
    filter = "all";
    index = 0;
    selected = null;
    listEl?.scrollTo(0, 0);
    reload();
  }

  $effect(() => {
    void search;
    void filter;
    const timer = setTimeout(reload, 80);
    return () => clearTimeout(timer);
  });

  onMount(() => {
    const un = listen("clips-changed", () => reload());
    return () => {
      un.then((f) => f());
    };
  });

  function select(i: number) {
    if (!items.length) return;
    index = Math.max(0, Math.min(items.length - 1, i));
    selected = items[index];
    listEl?.querySelector(`[data-i="${index}"]`)?.scrollIntoView({ block: "nearest" });
    if (index > items.length - 20) loadMore();
  }

  async function togglePin(c: Clip) {
    await api.clipPin(c.id, !c.pinned);
  }

  async function remove(c: Clip) {
    // 删掉后 reload 会选中同一位置的下一条，连续按删除键可以一路删下去
    await api.clipRemove(c.id);
  }

  function onKey(e: KeyboardEvent) {
    const cur = items[index];
    if (e.key === "ArrowDown") select(index + 1);
    else if (e.key === "ArrowUp") select(index - 1);
    else if (e.key === "Enter" && cur) onActivate(cur);
    else if (e.key === "Tab") {
      const i = FILTERS.findIndex((f) => f.id === filter);
      filter = FILTERS[(i + (e.shiftKey ? FILTERS.length - 1 : 1)) % FILTERS.length].id;
    } else if (modKey(e) && e.key.toLowerCase() === "p" && cur) togglePin(cur);
    else if (modKey(e) && e.key === "Backspace" && cur) remove(cur);
    else if (compact && modKey(e) && /^[1-9]$/.test(e.key) && items[Number(e.key) - 1]) onActivate(items[Number(e.key) - 1]);
    else return;
    e.preventDefault();
  }

  function firstLine(c: Clip) {
    if (c.kind === "files") {
      const paths = c.text.split("\n");
      const name = paths[0].split(/[\\/]/).pop() || paths[0];
      return paths.length > 1 ? t("{0} 等 {1} 个文件", name, paths.length) : name;
    }
    return c.text.replace(/^\s+/, "").slice(0, 400);
  }
</script>

<div class="clips" class:compact>
  <div class="search">
    <Search size={14} />
    <input bind:this={input} bind:value={search} onkeydown={onKey} placeholder={t("搜索剪贴板历史")} spellcheck="false" />
  </div>
  <div class="filters">
    {#each FILTERS as f (f.id)}
      <button class:on={filter === f.id} onclick={() => (filter = f.id)}>{t(f.name)}</button>
    {/each}
  </div>
  <div
    class="list"
    bind:this={listEl}
    onscroll={(e) => {
      const el = e.currentTarget;
      if (el.scrollTop + el.clientHeight > el.scrollHeight - 200) loadMore();
    }}
  >
    {#each items as c, i (c.id)}
      {@const kinds = c.kind === "text" ? detect(c.text).slice(0, 2) : []}
      <!-- svelte-ignore a11y_click_events_have_key_events -->
      <div
        class="item"
        class:on={i === index}
        data-i={i}
        role="option"
        tabindex="-1"
        aria-selected={i === index}
        onclick={() => select(i)}
        ondblclick={() => onActivate(c)}
      >
        <div class="kind">
          {#if c.kind === "image"}<Image size={14} />{:else if c.kind === "files"}<File size={14} />{:else}<Type size={14} />{/if}
        </div>
        <div class="main">
          {#if c.kind === "image"}
            <ClipThumb clip={c} />
          {:else}
            <div class="text" class:mono={kinds.length > 0}>{firstLine(c)}</div>
          {/if}
          <div class="meta">
            {#if c.pinned}<span class="pin"><Pin size={11} /></span>{/if}
            {#each kinds as k (k.tool + k.label)}<span class="badge">{t(k.label)}</span>{/each}
            {#if c.kind === "image"}<span>{c.width} × {c.height}</span>{/if}
            {#if c.source}<span class="source">{c.source}</span>{/if}
            <span class="time">{relativeTime(c.time, t)}</span>
            {#if compact && i < 9}<kbd>{i + 1}</kbd>{/if}
          </div>
        </div>
      </div>
    {:else}
      <div class="empty">{search || filter !== "all" ? t("没有匹配的记录") : t("还没有记录。复制点什么试试")}</div>
    {/each}
  </div>
</div>

<style>
  .clips {
    display: flex;
    flex-direction: column;
    height: 100%;
    min-height: 0;
  }
  .search {
    display: flex;
    align-items: center;
    gap: 8px;
    margin: 10px 10px 6px;
    padding: 0 10px;
    border: 1px solid var(--border);
    border-radius: 7px;
    background: var(--input-bg);
    color: var(--text-faint);
  }
  .search:focus-within {
    border-color: var(--accent);
  }
  .search input {
    flex: 1;
    height: 32px;
    padding: 0;
    border: 0;
    background: transparent;
  }
  .filters {
    display: flex;
    gap: 2px;
    padding: 0 10px 8px;
    border-bottom: 1px solid var(--border);
  }
  .filters button {
    height: 24px;
    padding: 0 10px;
    border: 0;
    border-radius: 12px;
    background: transparent;
    color: var(--text-dim);
    font-size: 12px;
  }
  .filters button.on {
    background: var(--bg-selected);
    color: var(--text);
  }
  .list {
    flex: 1;
    overflow-y: auto;
    padding: 4px 6px;
  }
  .item {
    display: flex;
    gap: 10px;
    padding: 8px 10px;
    border-radius: 6px;
    cursor: default;
  }
  .item:hover {
    background: var(--bg-hover);
  }
  .item.on {
    background: var(--bg-selected);
  }
  .kind {
    flex: 0 0 auto;
    padding-top: 2px;
    color: var(--text-faint);
  }
  .main {
    flex: 1;
    min-width: 0;
  }
  .text {
    display: -webkit-box;
    -webkit-line-clamp: 3;
    line-clamp: 3;
    -webkit-box-orient: vertical;
    overflow: hidden;
    white-space: pre-wrap;
    word-break: break-all;
    line-height: 1.4;
  }
  .compact .text {
    -webkit-line-clamp: 2;
    line-clamp: 2;
  }
  .text.mono {
    font-family: var(--mono);
    font-size: 12px;
  }
  .meta {
    display: flex;
    align-items: center;
    gap: 6px;
    margin-top: 4px;
    color: var(--text-faint);
    font-size: 11px;
  }
  .badge {
    padding: 0 6px;
    border-radius: 8px;
    background: var(--bg-hover);
    color: var(--accent);
  }
  .pin {
    display: inline-flex;
    color: var(--warn);
  }
  .source {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .time {
    margin-left: auto;
    white-space: nowrap;
  }
  .meta kbd {
    min-width: 16px;
    padding: 0 4px;
    font-size: 10px;
  }
  .empty {
    padding: 40px 20px;
    color: var(--text-faint);
    text-align: center;
  }
</style>
