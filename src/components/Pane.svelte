<!-- 工具里的一块输入或输出区域：标题栏（左边名字，右边按钮）加内容 -->
<script lang="ts">
  import type { Snippet } from "svelte";

  let { label, actions, children, grow = true }: { label: string; actions?: Snippet; children: Snippet; grow?: boolean } = $props();
</script>

<section class="pane" class:grow>
  <header>
    <span class="label">{label}</span>
    <div class="actions">{@render actions?.()}</div>
  </header>
  <div class="body">{@render children()}</div>
</section>

<style>
  .pane {
    display: flex;
    flex-direction: column;
    min-width: 0;
    min-height: 0;
  }
  .pane.grow {
    flex: 1;
  }
  header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
    min-height: 30px;
    margin-bottom: 6px;
  }
  .label {
    color: var(--text-dim);
    font-size: 12px;
    font-weight: 600;
  }
  .actions {
    display: flex;
    align-items: center;
    gap: 6px;
  }
  .actions :global(button) {
    height: 26px;
    padding: 0 9px;
    font-size: 12px;
  }
  .body {
    flex: 1;
    display: flex;
    flex-direction: column;
    min-height: 0;
  }
  .body > :global(textarea) {
    flex: 1;
    width: 100%;
    min-height: 80px;
  }
</style>
