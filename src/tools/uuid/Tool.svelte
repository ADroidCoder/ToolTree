<script lang="ts">
  import { untrack } from "svelte";
  import { RefreshCw } from "@lucide/svelte";
  import Pane from "../../components/Pane.svelte";
  import Seg from "../../components/Seg.svelte";
  import CopyButton from "../../components/CopyButton.svelte";
  import ResultRow from "../../components/ResultRow.svelte";
  import { t } from "../../lib/i18n.svelte";
  import { persisted } from "../../lib/persist.svelte";
  import { formatDateTime } from "../../lib/util";
  import { nanoid, parseId, ulid, uuidV4, uuidV7 } from "./ids";
  import type { ToolProps } from "../types";

  let { input }: ToolProps = $props();

  type Kind = "v4" | "v7" | "ulid" | "nanoid";
  const kind = persisted<Kind>("uuid.kind", "v4");
  const count = persisted("uuid.count", 5);
  const upper = persisted("uuid.upper", false);
  const noDash = persisted("uuid.noDash", false);
  const parseText = persisted("uuid.parse", "");
  const init = untrack(() => input);
  if (init !== undefined) parseText.value = init;

  let seed = $state(0);
  const ids = $derived.by(() => {
    void seed;
    const n = Math.max(1, Math.min(1000, Number(count.value) || 1));
    const gen: Record<Kind, () => string> = { v4: uuidV4, v7: () => uuidV7(), ulid: () => ulid(), nanoid: () => nanoid() };
    return Array.from({ length: n }, () => {
      let s = gen[kind.value]();
      if (kind.value === "v4" || kind.value === "v7") {
        if (noDash.value) s = s.replace(/-/g, "");
        if (upper.value) s = s.toUpperCase();
      }
      return s;
    });
  });
  const output = $derived(ids.join("\n"));
  const info = $derived(parseText.value.trim() ? parseId(parseText.value) : null);
</script>

<div class="tool">
  <div class="tool-bar">
    <Seg
      bind:value={kind.value}
      options={[
        { value: "v4", label: "UUID v4" },
        { value: "v7", label: "UUID v7" },
        { value: "ulid", label: "ULID" },
        { value: "nanoid", label: "NanoID" },
      ]}
    />
    <label class="check">{t("数量")}<input type="number" min="1" max="1000" bind:value={count.value} style="width: 72px" /></label>
    {#if kind.value === "v4" || kind.value === "v7"}
      <label class="check"><input type="checkbox" bind:checked={upper.value} />{t("大写")}</label>
      <label class="check"><input type="checkbox" bind:checked={noDash.value} />{t("去掉连字符")}</label>
    {/if}
  </div>
  <Pane label={t("生成结果")}>
    {#snippet actions()}
      <button class="icon" onclick={() => seed++}><RefreshCw size={13} />{t("重新生成")}</button>
      <CopyButton text={output} />
    {/snippet}
    <textarea class="code" readonly value={output}></textarea>
  </Pane>
  <div>
    <div class="tool-section-title">{t("解析")}</div>
    <div class="tool-bar">
      <input class="mono grow" bind:value={parseText.value} placeholder={t("粘贴 UUID 或 ULID")} />
    </div>
    {#if parseText.value.trim()}
      <div class="tool-rows" style="margin-top: 8px">
        {#if info}
          <ResultRow label={t("类型")} value={info.version ? `${info.kind} v${info.version}` : info.kind} />
          {#if info.time !== undefined}
            <ResultRow label={t("生成时间")} value={formatDateTime(info.time, true)} />
          {/if}
        {:else}
          <div class="tool-error">{t("不是有效的 UUID 或 ULID")}</div>
        {/if}
      </div>
    {/if}
  </div>
</div>

<style>
  .grow {
    flex: 1;
  }
</style>
