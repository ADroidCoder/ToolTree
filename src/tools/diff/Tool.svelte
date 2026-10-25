<script lang="ts">
  import { untrack } from "svelte";
  import { diffChars, diffLines, diffWordsWithSpace, type ChangeObject } from "diff";
  import { ArrowLeftRight } from "@lucide/svelte";
  import Pane from "../../components/Pane.svelte";
  import Seg from "../../components/Seg.svelte";
  import { t } from "../../lib/i18n.svelte";
  import { persisted } from "../../lib/persist.svelte";
  import type { ToolProps } from "../types";

  let { input }: ToolProps = $props();

  const left = persisted("diff.left", "");
  const right = persisted("diff.right", "");
  const mode = persisted<"lines" | "words" | "chars">("diff.mode", "lines");
  const ignoreSpace = persisted("diff.ignoreSpace", false);
  const ignoreCase = persisted("diff.ignoreCase", false);
  const init = untrack(() => input);
  if (init !== undefined) left.value = init;

  interface Line {
    kind: "same" | "add" | "del";
    text: string;
    a?: number;
    b?: number;
    /** 修改行里真正变了的片段 */
    parts?: ChangeObject<string>[];
  }

  /** 按行对比，删除和新增成对出现时再按词对比，标出行内具体改了哪里 */
  const lines = $derived.by((): Line[] => {
    if (mode.value !== "lines") return [];
    const changes = diffLines(left.value, right.value, { ignoreWhitespace: ignoreSpace.value, stripTrailingCr: true });
    const out: Line[] = [];
    let a = 1;
    let b = 1;
    const split = (s: string) => s.replace(/\n$/, "").split("\n");
    for (let i = 0; i < changes.length; i++) {
      const c = changes[i];
      const next = changes[i + 1];
      if (c.removed && next?.added) {
        const dl = split(c.value);
        const al = split(next.value);
        const n = Math.max(dl.length, al.length);
        const dels: Line[] = [];
        const adds: Line[] = [];
        for (let j = 0; j < n; j++) {
          const parts = dl[j] !== undefined && al[j] !== undefined ? diffWordsWithSpace(dl[j], al[j]) : undefined;
          if (dl[j] !== undefined) dels.push({ kind: "del", text: dl[j], a: a++, parts: parts?.filter((p) => !p.added) });
          if (al[j] !== undefined) adds.push({ kind: "add", text: al[j], b: b++, parts: parts?.filter((p) => !p.removed) });
        }
        out.push(...dels, ...adds);
        i++;
        continue;
      }
      for (const text of split(c.value)) {
        if (c.added) out.push({ kind: "add", text, b: b++ });
        else if (c.removed) out.push({ kind: "del", text, a: a++ });
        else out.push({ kind: "same", text, a: a++, b: b++ });
      }
    }
    return out;
  });

  const inline = $derived(
    mode.value === "lines" ? [] : (mode.value === "words" ? diffWordsWithSpace : diffChars)(left.value, right.value, { ignoreCase: ignoreCase.value }),
  );

  const stats = $derived(
    mode.value === "lines"
      ? { add: lines.filter((l) => l.kind === "add").length, del: lines.filter((l) => l.kind === "del").length }
      : { add: inline.filter((p) => p.added).length, del: inline.filter((p) => p.removed).length },
  );

  function swap() {
    [left.value, right.value] = [right.value, left.value];
  }
</script>

<div class="tool">
  <div class="tool-cols inputs">
    <Pane label={t("原文本")}>
      <textarea class="code" spellcheck="false" bind:value={left.value}></textarea>
    </Pane>
    <button class="ghost icon swap" onclick={swap} title={t("交换")}><ArrowLeftRight size={15} /></button>
    <Pane label={t("新文本")}>
      <textarea class="code" spellcheck="false" bind:value={right.value}></textarea>
    </Pane>
  </div>
  <div class="tool-bar">
    <Seg bind:value={mode.value} options={[{ value: "lines", label: t("按行") }, { value: "words", label: t("按词") }, { value: "chars", label: t("按字符") }]} />
    {#if mode.value === "lines"}
      <label class="check"><input type="checkbox" bind:checked={ignoreSpace.value} />{t("忽略空白")}</label>
    {:else}
      <label class="check"><input type="checkbox" bind:checked={ignoreCase.value} />{t("忽略大小写")}</label>
    {/if}
    <span class="stats"><span class="add">+{stats.add}</span> <span class="del">-{stats.del}</span></span>
  </div>
  <div class="result mono">
    {#if mode.value === "lines"}
      {#each lines as l, i (i)}
        <div class="line {l.kind}">
          <span class="no">{l.a ?? ""}</span><span class="no">{l.b ?? ""}</span><span class="sign">{l.kind === "add" ? "+" : l.kind === "del" ? "-" : ""}</span>
          <span class="text">{#if l.parts}{#each l.parts as p, k (k)}<span class:strong={p.added || p.removed}>{p.value}</span>{/each}{:else}{l.text}{/if}</span>
        </div>
      {/each}
    {:else}
      <div class="inline">{#each inline as p, i (i)}<span class:ins={p.added} class:rm={p.removed}>{p.value}</span>{/each}</div>
    {/if}
  </div>
</div>

<style>
  .inputs {
    flex: 0 0 38%;
    align-items: stretch;
  }
  .swap {
    align-self: center;
    padding: 0 6px;
  }
  .stats {
    margin-left: auto;
    font-weight: 600;
  }
  .add {
    color: var(--add);
  }
  .del {
    color: var(--del);
  }
  .result {
    flex: 1;
    min-height: 0;
    overflow: auto;
    border: 1px solid var(--border);
    border-radius: var(--radius);
    background: var(--bg-card);
    user-select: text;
    -webkit-user-select: text;
  }
  .line {
    display: flex;
    min-width: max-content;
    line-height: 20px;
  }
  .line.add {
    background: var(--bg-add);
  }
  .line.del {
    background: var(--bg-del);
  }
  .no {
    flex: 0 0 42px;
    padding-right: 8px;
    color: var(--text-faint);
    text-align: right;
    user-select: none;
  }
  .sign {
    flex: 0 0 18px;
    color: var(--text-faint);
    text-align: center;
    user-select: none;
  }
  .text {
    white-space: pre;
    padding-right: 12px;
  }
  .line.add .strong {
    background: var(--bg-add-strong);
  }
  .line.del .strong {
    background: var(--bg-del-strong);
  }
  .inline {
    padding: 10px 12px;
    white-space: pre-wrap;
    word-break: break-word;
    line-height: 1.6;
  }
  .ins {
    background: var(--bg-add-strong);
  }
  .rm {
    background: var(--bg-del-strong);
    text-decoration: line-through;
  }
</style>
