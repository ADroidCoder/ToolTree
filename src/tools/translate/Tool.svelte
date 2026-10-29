<script lang="ts">
  import { onMount, untrack } from "svelte";
  import { ArrowLeftRight } from "@lucide/svelte";
  import Pane from "../../components/Pane.svelte";
  import CopyButton from "../../components/CopyButton.svelte";
  import { api, type Config } from "../../lib/api";
  import { t } from "../../lib/i18n.svelte";
  import { persisted } from "../../lib/persist.svelte";
  import { LANGS, PROVIDERS, autoTarget } from "./translate";
  import type { ToolProps } from "../types";

  let { input }: ToolProps = $props();

  const text = persisted("translate.input", "");
  const from = persisted("translate.from", "auto");
  const to = persisted("translate.to", "auto");
  const init = untrack(() => input);
  if (init !== undefined) text.value = init;

  let config = $state<Config | null>(null);
  let out = $state("");
  let detected = $state("");
  let error = $state("");
  let busy = $state(false);

  onMount(async () => {
    config = await api.configGet();
  });

  const provider = $derived(config?.translate.provider || "free");
  const target = $derived(to.value === "auto" ? autoTarget(text.value, from.value) : to.value);
  const langName = (code: string) => t(LANGS.find((l) => l.value === code)?.label ?? code);

  async function setProvider(p: string) {
    if (!config) return;
    config = { ...config, translate: { ...config.translate, provider: p } };
    // 这里只改翻译服务，快捷键注册失败的提示留给设置页
    await api.configSet(config).catch(() => {});
  }

  /** 每次请求编个号，只用最后一次的结果，免得慢的旧结果把新的盖掉 */
  let seq = 0;
  async function run() {
    const s = text.value;
    const id = ++seq;
    if (!s.trim()) {
      out = detected = error = "";
      busy = false;
      return;
    }
    busy = true;
    try {
      const r = await api.translate(s, from.value, target);
      if (id !== seq) return;
      out = r.text;
      detected = r.from;
      error = "";
    } catch (e) {
      if (id !== seq) return;
      error = String(e);
    } finally {
      if (id === seq) busy = false;
    }
  }

  // 停止输入一会儿再翻译，按 ⌘/Ctrl+回车 立刻翻译
  let timer: ReturnType<typeof setTimeout> | undefined;
  $effect(() => {
    void [text.value, from.value, target, provider];
    if (!config) return;
    clearTimeout(timer);
    timer = setTimeout(run, 700);
    return () => clearTimeout(timer);
  });

  function onKey(e: KeyboardEvent) {
    if (e.key === "Enter" && (e.metaKey || e.ctrlKey)) {
      e.preventDefault();
      clearTimeout(timer);
      run();
    }
  }

  /** 交换语言，译文变成原文 */
  function swap() {
    const src = from.value === "auto" ? detected || autoSource() : from.value;
    from.value = target;
    to.value = src;
    if (out) text.value = out;
  }

  function autoSource() {
    return target === "zh" ? "en" : "zh";
  }
</script>

<div class="tool">
  <div class="tool-bar">
    <select bind:value={from.value}>
      <option value="auto">{t("自动识别")}</option>
      {#each LANGS as l (l.value)}<option value={l.value}>{t(l.label)}</option>{/each}
    </select>
    <button class="ghost icon" title={t("交换")} onclick={swap}><ArrowLeftRight size={14} /></button>
    <select bind:value={to.value}>
      <option value="auto">{t("自动（中英互译）")}</option>
      {#each LANGS as l (l.value)}<option value={l.value}>{t(l.label)}</option>{/each}
    </select>
    <span class="spacer"></span>
    {#if config}
      <select value={provider} onchange={(e) => setProvider(e.currentTarget.value)} title={t("翻译服务，密钥在设置里填")}>
        {#each PROVIDERS as p (p.value)}<option value={p.value}>{t(p.label)}</option>{/each}
      </select>
    {/if}
  </div>

  <div class="tool-cols">
    <Pane label={t("原文（{0}）", from.value !== "auto" ? langName(from.value) : detected ? langName(detected) : t("自动识别"))}>
      {#snippet actions()}
        <button onclick={() => (text.value = "")}>{t("清空")}</button>
      {/snippet}
      <textarea spellcheck="false" bind:value={text.value} onkeydown={onKey} placeholder={t("输入或粘贴要翻译的内容，⌘/Ctrl+回车立即翻译")}></textarea>
    </Pane>
    <Pane label={busy ? t("正在翻译…") : t("译文（{0}）", langName(target))}>
      {#snippet actions()}
        <CopyButton text={out} />
      {/snippet}
      {#if error}<div class="tool-error" style="margin-bottom: 6px">{error}</div>{/if}
      <textarea spellcheck="false" readonly value={out} class:stale={busy}></textarea>
    </Pane>
  </div>
</div>

<style>
  .spacer {
    flex: 1;
  }
  textarea {
    resize: none;
    line-height: 1.6;
  }
  .stale {
    opacity: 0.6;
  }
</style>
