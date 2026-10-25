<script lang="ts">
  import { untrack } from "svelte";
  import Pane from "../../components/Pane.svelte";
  import Seg from "../../components/Seg.svelte";
  import CopyButton from "../../components/CopyButton.svelte";
  import ResultRow from "../../components/ResultRow.svelte";
  import { t } from "../../lib/i18n.svelte";
  import { persisted } from "../../lib/persist.svelte";
  import type { ToolProps } from "../types";

  let { input }: ToolProps = $props();

  const mode = persisted<"decode" | "encode">("url.mode", "decode");
  /** component 会编码 / ? & = 这些字符，适合编码参数值；full 保留网址结构 */
  const scope = persisted<"component" | "full">("url.scope", "component");
  const text = persisted("url.input", "");
  const init = untrack(() => input);
  if (init !== undefined) text.value = init;

  const result = $derived.by((): { out: string; error?: string } => {
    const s = text.value;
    if (!s) return { out: "" };
    try {
      if (mode.value === "encode") return { out: scope.value === "full" ? encodeURI(s) : encodeURIComponent(s) };
      // 表单提交里 + 表示空格
      return { out: decodeURIComponent(s.replace(/\+/g, " ")) };
    } catch {
      return { out: "", error: t("包含无效的 % 编码") };
    }
  });

  /** 能当成网址解析时拆开显示 */
  const parsed = $derived.by(() => {
    const s = text.value.trim();
    if (!/^[a-z][\w+.-]*:\/\//i.test(s)) return null;
    try {
      return new URL(s);
    } catch {
      return null;
    }
  });
  const params = $derived(parsed ? [...parsed.searchParams.entries()] : []);
</script>

<div class="tool">
  <div class="tool-bar">
    <Seg bind:value={mode.value} options={[{ value: "decode", label: t("解码") }, { value: "encode", label: t("编码") }]} />
    {#if mode.value === "encode"}
      <Seg bind:value={scope.value} options={[{ value: "component", label: t("参数值（encodeURIComponent）") }, { value: "full", label: t("整个网址（encodeURI）") }]} />
    {/if}
  </div>
  <div class="tool-cols short">
    <Pane label={t("输入")}>
      {#snippet actions()}
        <button onclick={() => (text.value = "")}>{t("清空")}</button>
      {/snippet}
      <textarea class="code wrap" spellcheck="false" bind:value={text.value}></textarea>
    </Pane>
    <Pane label={t("结果")}>
      {#snippet actions()}
        <button disabled={!result.out} onclick={() => ((text.value = result.out), (mode.value = mode.value === "encode" ? "decode" : "encode"))}>{t("作为输入")}</button>
        <CopyButton text={result.out} />
      {/snippet}
      {#if result.error}<div class="tool-error" style="margin-bottom: 6px">{result.error}</div>{/if}
      <textarea class="code wrap" spellcheck="false" readonly value={result.out}></textarea>
    </Pane>
  </div>

  {#if parsed}
    <div class="parts">
      <div class="tool-section-title">{t("网址结构")}</div>
      <div class="tool-rows">
        <ResultRow label={t("协议")} value={parsed.protocol.replace(/:$/, "")} />
        <ResultRow label={t("主机")} value={parsed.hostname} />
        {#if parsed.port}<ResultRow label={t("端口")} value={parsed.port} />{/if}
        <ResultRow label={t("路径")} value={decodeSafe(parsed.pathname)} />
        {#if parsed.hash}<ResultRow label={t("锚点")} value={decodeSafe(parsed.hash)} />{/if}
      </div>
      {#if params.length}
        <div class="tool-section-title">{t("查询参数（{0}）", params.length)}</div>
        <div class="tool-rows">
          {#each params as [k, v], i (i)}
            <ResultRow label={k} value={v} />
          {/each}
        </div>
      {/if}
    </div>
  {/if}
</div>

<script lang="ts" module>
  function decodeSafe(s: string) {
    try {
      return decodeURIComponent(s);
    } catch {
      return s;
    }
  }
</script>

<style>
  .short {
    flex: 0 0 200px;
  }
  .wrap {
    white-space: pre-wrap;
    word-break: break-all;
  }
  .parts {
    flex: 1;
    overflow: auto;
    min-height: 0;
  }
</style>
