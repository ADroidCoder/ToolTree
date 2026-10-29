<!-- 录制全局快捷键：点一下进入录制，按下组合键完成，Esc 或点别处取消 -->
<script lang="ts">
  import { X } from "@lucide/svelte";
  import { api } from "../lib/api";
  import { display, fromEvent } from "../lib/shortcut";
  import { t } from "../lib/i18n.svelte";

  let { value, onchange }: { value: string; onchange: (v: string) => void } = $props();
  let recording = $state(false);
  let btn: HTMLButtonElement;

  function onKey(e: KeyboardEvent) {
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

  // macOS 的 WebView 里点按钮不会让按钮获得焦点，按键事件到不了按钮上，所以录制时在整个窗口上监听。
  // 同时暂停全局快捷键，否则按下已经注册的组合键（比如原来的截图快捷键）会被直接触发，这里收不到
  $effect(() => {
    if (!recording) return;
    const stop = () => (recording = false);
    const onDown = (e: MouseEvent) => {
      if (!btn.contains(e.target as Node)) stop();
    };
    api.shortcutsPause(true);
    addEventListener("keydown", onKey, true);
    addEventListener("mousedown", onDown, true);
    addEventListener("blur", stop);
    return () => {
      removeEventListener("keydown", onKey, true);
      removeEventListener("mousedown", onDown, true);
      removeEventListener("blur", stop);
      api.shortcutsPause(false);
    };
  });
</script>

<div class="sc">
  <button bind:this={btn} class="rec" class:on={recording} onclick={() => (recording = true)}>
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
