<script lang="ts">
  import { untrack } from "svelte";
  import yaml from "js-yaml";
  import Pane from "../../components/Pane.svelte";
  import Seg from "../../components/Seg.svelte";
  import CopyButton from "../../components/CopyButton.svelte";
  import { t } from "../../lib/i18n.svelte";
  import { persisted } from "../../lib/persist.svelte";
  import { parseJson, sortKeys, toTypeScript } from "./json";
  import type { ToolProps } from "../types";

  let { input }: ToolProps = $props();

  const text = persisted("json.input", '{"name":"ToolTree","tools":["json","base64"],"version":1}');
  const mode = persisted<"pretty" | "minify" | "yaml" | "ts">("json.mode", "pretty");
  const indent = persisted<"2" | "4" | "tab">("json.indent", "2");
  const sorted = persisted("json.sort", false);
  const init = untrack(() => input);
  if (init !== undefined) text.value = init;

  const parsed = $derived(text.value.trim() ? parseJson(text.value) : {});

  const output = $derived.by(() => {
    if (parsed.error || parsed.value === undefined) return "";
    const v = sorted.value ? sortKeys(parsed.value) : parsed.value;
    const ind = indent.value === "tab" ? "\t" : Number(indent.value);
    try {
      if (mode.value === "minify") return JSON.stringify(v);
      if (mode.value === "yaml") return yaml.dump(v, { indent: indent.value === "4" ? 4 : 2, lineWidth: -1, noRefs: true });
      if (mode.value === "ts") return toTypeScript(v);
      return JSON.stringify(v, null, ind);
    } catch (e) {
      return String(e);
    }
  });

  /** 输入是一个 JSON 字符串（比如日志里转义过的 "{\"a\":1}"）时解开一层 */
  function unescape() {
    const r = parseJson(text.value.trim());
    if (typeof r.value === "string") text.value = r.value;
    else if (!r.error) return;
    else {
      // 没有外层引号的转义内容
      const r2 = parseJson(`"${text.value.trim()}"`);
      if (typeof r2.value === "string") text.value = r2.value;
    }
  }

  function escape() {
    text.value = JSON.stringify(text.value);
  }

  function fromYaml() {
    try {
      text.value = JSON.stringify(yaml.load(text.value), null, 2);
    } catch (e) {
      parsedYamlError = String(e);
      setTimeout(() => (parsedYamlError = ""), 4000);
    }
  }
  let parsedYamlError = $state("");
</script>

<div class="tool">
  <div class="tool-bar">
    <Seg
      bind:value={mode.value}
      options={[
        { value: "pretty", label: t("格式化") },
        { value: "minify", label: t("压缩") },
        { value: "yaml", label: "YAML" },
        { value: "ts", label: "TypeScript" },
      ]}
    />
    {#if mode.value === "pretty" || mode.value === "yaml"}
      <Seg
        bind:value={indent.value}
        options={[
          { value: "2", label: t("2 空格") },
          { value: "4", label: t("4 空格") },
          ...(mode.value === "pretty" ? [{ value: "tab" as const, label: "Tab" }] : []),
        ]}
      />
    {/if}
    <label class="check"><input type="checkbox" bind:checked={sorted.value} />{t("按键名排序")}</label>
  </div>
  <div class="tool-cols">
    <Pane label={t("输入")}>
      {#snippet actions()}
        <button onclick={escape}>{t("转义")}</button>
        <button onclick={unescape}>{t("去转义")}</button>
        <button onclick={fromYaml}>{t("从 YAML 转换")}</button>
        <button onclick={() => (text.value = "")}>{t("清空")}</button>
      {/snippet}
      <textarea class="code" spellcheck="false" bind:value={text.value} placeholder={t("粘贴 JSON")}></textarea>
    </Pane>
    <Pane label={t("结果")}>
      {#snippet actions()}
        <CopyButton text={output} />
      {/snippet}
      {#if parsed.error || parsedYamlError}
        <div class="tool-error" style="margin-bottom: 6px">{parsedYamlError || parsed.error}</div>
      {/if}
      <textarea class="code" spellcheck="false" readonly value={output}></textarea>
    </Pane>
  </div>
</div>
