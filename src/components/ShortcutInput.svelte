<!-- 录制全局快捷键：点一下进入录制，按下组合键完成，Esc 取消 -->
<script lang="ts">
  import { X } from "@lucide/svelte";
  import { display, fromEvent } from "../lib/shortcut";
  import { t } from "../lib/i18n.svelte";

  let { value, onchange }: { value: string; onchange: (v: string) => void } = $props();
  let recording = $state(false);

  function onKey(e: KeyboardEvent) {
    if (!recording) return;
    e.preventDefault();
    e.stopPropagation();
    if (e.key === "Escape") {
      recording = false;
      return;
    }
    const s = fromEvent(e);
    if (s) {
      recording = false;
      onchange(s);
    }
  }
</script>

<div class="sc">
  <button class="rec" class:on={recording} onclick={() => (recording = true)} onkeydown={onKey} onblur={() => (recording = false)}>
    {recording ? t("按下快捷键…") : display(value) || t("未设置")}
  </button>
  {#if value}
    <button class="ghost icon clear" title={t("清除")} onclick={() => onchange("")}><X size={13} /></button>
  {/if}
</div>

<style>
  .sc {
    display: inline-flex;
    align-items: center;
    gap: 4px;
  }
  .rec {
    min-width: 140px;
    font-family: var(--mono);
  }
  .rec.on {
    border-color: var(--accent);
    color: var(--accent);
  }
  .clear {
    padding: 0 6px;
    color: var(--text-faint);
  }
</style>
