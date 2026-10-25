<!-- 分段选择：几个互斥的选项排成一行 -->
<script lang="ts" generics="T extends string | number">
  let { options, value = $bindable() }: { options: { value: T; label: string }[]; value: T } = $props();
</script>

<div class="seg">
  {#each options as o (o.value)}
    <button class:on={o.value === value} onclick={() => (value = o.value)}>{o.label}</button>
  {/each}
</div>

<style>
  .seg {
    display: inline-flex;
    padding: 2px;
    border: 1px solid var(--border);
    border-radius: 6px;
    background: var(--input-bg);
  }
  button {
    height: 24px;
    padding: 0 10px;
    border: 0;
    border-radius: 4px;
    background: transparent;
    color: var(--text-dim);
    font-size: 12px;
  }
  button.on {
    background: var(--accent);
    color: #fff;
  }
</style>
