<script lang="ts">
  import { untrack } from "svelte";
  import Pane from "../../components/Pane.svelte";
  import ResultRow from "../../components/ResultRow.svelte";
  import { t } from "../../lib/i18n.svelte";
  import { persisted } from "../../lib/persist.svelte";
  import { CASES, stats, words } from "./case";
  import type { ToolProps } from "../types";

  let { input }: ToolProps = $props();

  const text = persisted("case.input", "user profile id");
  const init = untrack(() => input);
  if (init !== undefined) text.value = init;

  /** 多行时每行分别转换 */
  const lines = $derived(text.value.split("\n"));
  const st = $derived(stats(text.value));
</script>

<div class="tool">
  <Pane label={t("输入")} grow={false}>
    {#snippet actions()}
      <button onclick={() => (text.value = text.value.toUpperCase())}>{t("全部大写")}</button>
      <button onclick={() => (text.value = text.value.toLowerCase())}>{t("全部小写")}</button>
      <button onclick={() => (text.value = [...new Set(lines)].join("\n"))}>{t("按行去重")}</button>
      <button onclick={() => (text.value = [...lines].sort((a, b) => a.localeCompare(b)).join("\n"))}>{t("按行排序")}</button>
      <button onclick={() => (text.value = lines.map((l) => l.trim()).filter(Boolean).join("\n"))}>{t("去掉空行")}</button>
    {/snippet}
    <textarea class="code" rows="5" spellcheck="false" bind:value={text.value}></textarea>
  </Pane>
  <div class="stats">
    <span>{t("字符")} <b>{st.chars}</b></span>
    <span>{t("字节")} <b>{st.bytes}</b></span>
    <span>{t("行")} <b>{st.lines}</b></span>
    <span>{t("英文单词")} <b>{st.words}</b></span>
    <span>{t("汉字")} <b>{st.cjk}</b></span>
  </div>
  <div class="tool-rows list">
    {#each CASES as c (c.name)}
      <ResultRow label={c.name} value={lines.map((l) => c.fn(words(l))).join("\n")} />
    {/each}
  </div>
</div>

<style>
  .stats {
    display: flex;
    flex-wrap: wrap;
    gap: 18px;
    color: var(--text-dim);
    font-size: 12px;
  }
  .stats b {
    color: var(--text);
    font-weight: 600;
  }
  .list {
    overflow: auto;
    min-height: 0;
  }
</style>
