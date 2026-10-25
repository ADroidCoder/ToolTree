<script lang="ts">
  import { untrack } from "svelte";
  import Seg from "../../components/Seg.svelte";
  import ResultRow from "../../components/ResultRow.svelte";
  import { t } from "../../lib/i18n.svelte";
  import { persisted } from "../../lib/persist.svelte";
  import type { ToolProps } from "../types";

  let { input }: ToolProps = $props();

  const text = persisted("radix.input", "255");
  const base = persisted("radix.base", 10);
  const init = untrack(() => input);
  if (init !== undefined) {
    const s = init.trim().toLowerCase();
    text.value = s.replace(/^0[xbo]/, "");
    base.value = s.startsWith("0x") ? 16 : s.startsWith("0b") ? 2 : s.startsWith("0o") ? 8 : 10;
  }

  const DIGITS = "0123456789abcdefghijklmnopqrstuvwxyz";

  /** 用 BigInt 转换，超过 2^53 的数也不会丢精度 */
  function parse(s: string, b: number): bigint | null {
    let v = s.trim().toLowerCase().replace(/[_\s]/g, "");
    if (!v) return null;
    const neg = v.startsWith("-");
    if (neg) v = v.slice(1);
    if (b === 16) v = v.replace(/^0x/, "");
    if (b === 2) v = v.replace(/^0b/, "");
    if (b === 8) v = v.replace(/^0o/, "");
    let n = 0n;
    for (const ch of v) {
      const d = DIGITS.indexOf(ch);
      if (d < 0 || d >= b) return null;
      n = n * BigInt(b) + BigInt(d);
    }
    return neg ? -n : n;
  }

  const custom = persisted("radix.custom", 36);
  const value = $derived(parse(text.value, Number(base.value)));

  /** 二进制每 4 位加一个空格，好数 */
  const group = (s: string, n: number) => s.replace(new RegExp(`\\B(?=(.{${n}})+$)`, "g"), " ");
</script>

<div class="tool">
  <div class="tool-bar">
    <span class="dim">{t("输入进制")}</span>
    <Seg bind:value={base.value} options={[{ value: 2, label: "2" }, { value: 8, label: "8" }, { value: 10, label: "10" }, { value: 16, label: "16" }]} />
    <input class="mono grow" bind:value={text.value} spellcheck="false" />
  </div>
  {#if text.value.trim() && value === null}
    <div class="tool-error">{t("不是有效的 {0} 进制数", base.value)}</div>
  {:else if value !== null}
    <div class="tool-rows">
      <ResultRow label={t("二进制")} value={value.toString(2)} />
      <ResultRow label={t("二进制（分组）")} value={group(value.toString(2), 4)} />
      <ResultRow label={t("八进制")} value={value.toString(8)} />
      <ResultRow label={t("十进制")} value={value.toString(10)} />
      <ResultRow label={t("十进制（千分位）")} value={group(value.toString(10), 3).replace(/ /g, ",")} />
      <ResultRow label={t("十六进制")} value={value.toString(16).toUpperCase()} />
      <ResultRow label={t("{0} 进制", custom.value)} value={value.toString(Math.max(2, Math.min(36, Number(custom.value) || 36)))} />
    </div>
    <label class="check">{t("自定义进制")}<input type="number" min="2" max="36" bind:value={custom.value} style="width: 70px" /></label>
  {/if}
</div>

<style>
  .grow {
    flex: 1;
  }
</style>
