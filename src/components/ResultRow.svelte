<!-- 一行结果：左边名字，中间值（等宽字体，可选中），右边复制按钮 -->
<script lang="ts">
  import type { Snippet } from "svelte";
  import CopyButton from "./CopyButton.svelte";

  let { label, value, extra }: { label: string; value: string; extra?: Snippet } = $props();
</script>

<div class="row">
  <span class="label">{label}</span>
  <span class="value mono" title={value}>{value || "—"}</span>
  {@render extra?.()}
  <CopyButton text={value} label={false} />
</div>

<style>
  .row {
    display: flex;
    align-items: center;
    gap: 10px;
    min-height: 34px;
    padding: 3px 4px 3px 12px;
    border: 1px solid var(--border);
    border-radius: 6px;
    background: var(--bg-card);
  }
  .label {
    flex: 0 0 110px;
    color: var(--text-dim);
    font-size: 12px;
  }
  .value {
    flex: 1;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    user-select: text;
    -webkit-user-select: text;
  }
  .row :global(button) {
    height: 26px;
    padding: 0 8px;
  }
</style>
