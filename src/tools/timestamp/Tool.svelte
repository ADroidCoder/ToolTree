<script lang="ts">
  import { onMount, untrack } from "svelte";
  import { Pause, Play } from "@lucide/svelte";
  import ResultRow from "../../components/ResultRow.svelte";
  import CopyButton from "../../components/CopyButton.svelte";
  import { t, i18n } from "../../lib/i18n.svelte";
  import { persisted } from "../../lib/persist.svelte";
  import { formatInZone, parseInput, relativeOffset, zoneOffset } from "./time";
  import type { ToolProps } from "../types";

  let { input }: ToolProps = $props();

  const local = Intl.DateTimeFormat().resolvedOptions().timeZone;
  const ZONES = [local, "UTC", "Asia/Shanghai", "Asia/Tokyo", "Asia/Singapore", "Europe/London", "Europe/Berlin", "America/New_York", "America/Los_Angeles"]
    .filter((z, i, a) => a.indexOf(z) === i);

  const text = persisted("timestamp.input", "");
  const zone = persisted("timestamp.zone", local);
  const init = untrack(() => input);
  if (init !== undefined) text.value = init;

  let now = $state(Date.now());
  let running = $state(true);
  onMount(() => {
    const timer = setInterval(() => running && (now = Date.now()), 1000);
    return () => clearInterval(timer);
  });

  const parsed = $derived(parseInput(text.value));
  const ms = $derived(parsed?.ms ?? null);
  const unitName: Record<string, string> = { s: "秒", ms: "毫秒", us: "微秒", ns: "纳秒" };
  const weekday = $derived(
    ms === null ? "" : new Intl.DateTimeFormat(i18n.lang === "en" ? "en-US" : "zh-CN", { weekday: "long", timeZone: zone.value }).format(new Date(ms)),
  );
</script>

<div class="tool">
  <div class="now">
    <div class="cell">
      <span class="faint">{t("当前时间戳（秒）")}</span>
      <span class="big mono">{Math.floor(now / 1000)}</span>
      <CopyButton text={String(Math.floor(now / 1000))} label={false} />
    </div>
    <div class="cell">
      <span class="faint">{t("毫秒")}</span>
      <span class="big mono">{now}</span>
      <CopyButton text={String(now)} label={false} />
    </div>
    <div class="cell">
      <span class="big mono small">{formatInZone(now, zone.value, false)}</span>
    </div>
    <button class="ghost icon" onclick={() => (running = !running)} title={running ? t("暂停") : t("继续")}>
      {#if running}<Pause size={14} />{:else}<Play size={14} />{/if}
    </button>
  </div>

  <div class="tool-bar">
    <input class="mono grow" bind:value={text.value} placeholder={t("输入时间戳或日期，比如 1761400000、2026-10-25 12:00:00")} />
    <button onclick={() => (text.value = String(Math.floor(Date.now() / 1000)))}>{t("现在")}</button>
    <select bind:value={zone.value}>
      {#each ZONES as z (z)}
        <option value={z}>{z === local ? t("本地（{0}）", z) : z} {zoneOffset(now, z)}</option>
      {/each}
    </select>
  </div>

  {#if text.value.trim() && ms === null}
    <div class="tool-error">{t("无法识别的时间格式")}</div>
  {:else if ms !== null}
    <div class="tool-rows">
      {#if parsed?.unit}
        <div class="faint hint">{t("识别为{0}级时间戳", t(unitName[parsed.unit]))}</div>
      {/if}
      <ResultRow label={t("日期时间")} value={formatInZone(ms, zone.value)} />
      <ResultRow label={t("秒")} value={String(Math.floor(ms / 1000))} />
      <ResultRow label={t("毫秒")} value={String(ms)} />
      <ResultRow label="ISO 8601" value={new Date(ms).toISOString()} />
      <ResultRow label="RFC 2822" value={new Date(ms).toUTCString()} />
      <ResultRow label={t("星期")} value={weekday} />
      <ResultRow label={t("相对现在")} value={relativeOffset(ms, now, t)} />
    </div>
  {/if}
</div>


<style>
  .now {
    display: flex;
    align-items: center;
    gap: 22px;
    padding: 14px 16px;
    border: 1px solid var(--border);
    border-radius: var(--radius);
    background: var(--bg-card);
  }
  .cell {
    display: flex;
    align-items: center;
    gap: 8px;
  }
  .cell :global(button) {
    height: 24px;
    padding: 0 7px;
  }
  .big {
    font-size: 17px;
    font-weight: 600;
  }
  .big.small {
    font-size: 14px;
    font-weight: 500;
    color: var(--text-dim);
  }
  .now > button {
    margin-left: auto;
  }
  .grow {
    flex: 1;
    min-width: 200px;
  }
  .hint {
    font-size: 12px;
    padding-left: 2px;
  }
</style>
