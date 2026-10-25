<script lang="ts">
  import { untrack } from "svelte";
  import ResultRow from "../../components/ResultRow.svelte";
  import { t } from "../../lib/i18n.svelte";
  import { persisted } from "../../lib/persist.svelte";
  import { contrast, parseColor, toHex, toHsl, toHsv } from "./color";
  import type { ToolProps } from "../types";

  let { input }: ToolProps = $props();

  const text = persisted("color.input", "#4d88ff");
  const init = untrack(() => input);
  if (init !== undefined) text.value = init;

  const c = $derived(parseColor(text.value));
  const hsl = $derived(c && toHsl(c));
  const hsv = $derived(c && toHsv(c));
  const rows = $derived(
    c && hsl && hsv
      ? [
          ["HEX", toHex(c)],
          ...(c.a < 1 ? [["HEX8", toHex(c, true)]] : []),
          ["RGB", c.a < 1 ? `rgba(${c.r}, ${c.g}, ${c.b}, ${c.a})` : `rgb(${c.r}, ${c.g}, ${c.b})`],
          ["HSL", c.a < 1 ? `hsla(${hsl.h}, ${hsl.s}%, ${hsl.l}%, ${c.a})` : `hsl(${hsl.h}, ${hsl.s}%, ${hsl.l}%)`],
          ["HSV", `hsv(${hsv.h}, ${hsv.s}%, ${hsv.v}%)`],
          ["Android", "#" + Math.round(c.a * 255).toString(16).padStart(2, "0") + toHex(c).slice(1)],
          ["Flutter", "Color(0x" + (Math.round(c.a * 255).toString(16).padStart(2, "0") + toHex(c).slice(1)).toUpperCase() + ")"],
          ["SwiftUI", `Color(red: ${(c.r / 255).toFixed(3)}, green: ${(c.g / 255).toFixed(3)}, blue: ${(c.b / 255).toFixed(3)})`],
        ]
      : [],
  );

  const white = { r: 255, g: 255, b: 255, a: 1 };
  const black = { r: 0, g: 0, b: 0, a: 1 };
  /** 对比度等级：4.5 以上 AA，7 以上 AAA */
  const grade = (n: number) => (n >= 7 ? "AAA" : n >= 4.5 ? "AA" : n >= 3 ? t("AA 大号字") : t("不合格"));
</script>

<div class="tool">
  <div class="tool-bar">
    <input type="color" value={c ? toHex(c) : "#000000"} oninput={(e) => (text.value = e.currentTarget.value)} />
    <input class="mono grow" bind:value={text.value} placeholder={t("#ff6600、rgb(255, 102, 0)、hsl(24, 100%, 50%)、orange")} />
  </div>
  {#if !c}
    {#if text.value.trim()}<div class="tool-error">{t("无法识别的颜色")}</div>{/if}
  {:else}
    <div class="preview">
      <div class="swatch" style:background={`rgba(${c.r}, ${c.g}, ${c.b}, ${c.a})`}></div>
      <div class="contrast">
        {#each [[white, "#fff", t("白色文字")], [black, "#000", t("黑色文字")]] as const as [bg, fg, name] (fg)}
          {@const n = contrast(c, bg)}
          <div class="sample" style:background={toHex(c)} style:color={fg}>
            <b>Aa</b> {name}
            <span class="ratio">{n.toFixed(2)} · {grade(n)}</span>
          </div>
        {/each}
      </div>
    </div>
    <div class="tool-rows">
      {#each rows as [name, value] (name)}
        <ResultRow label={name} {value} />
      {/each}
    </div>
  {/if}
</div>

<style>
  .grow {
    flex: 1;
  }
  input[type="color"] {
    width: 42px;
    height: 30px;
    padding: 2px;
    border: 1px solid var(--border);
    border-radius: 5px;
    background: var(--input-bg);
    cursor: pointer;
  }
  .preview {
    display: flex;
    gap: 14px;
  }
  .swatch {
    flex: 0 0 140px;
    height: 96px;
    border-radius: var(--radius);
    border: 1px solid var(--border);
  }
  .contrast {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .sample {
    flex: 1;
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 0 14px;
    border-radius: var(--radius);
    font-size: 13px;
  }
  .sample b {
    font-size: 18px;
  }
  .ratio {
    margin-left: auto;
    font-size: 12px;
    opacity: 0.85;
  }
</style>
