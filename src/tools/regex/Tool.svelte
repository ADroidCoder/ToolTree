<script lang="ts">
  import { untrack } from "svelte";
  import Pane from "../../components/Pane.svelte";
  import CopyButton from "../../components/CopyButton.svelte";
  import { t } from "../../lib/i18n.svelte";
  import { persisted } from "../../lib/persist.svelte";
  import type { ToolProps } from "../types";

  let { input }: ToolProps = $props();

  const pattern = persisted("regex.pattern", "\\b(\\w+)@(\\w+)\\.com\\b");
  const flags = persisted("regex.flags", "g");
  const text = persisted("regex.text", "联系我们：hello@example.com, admin@test.com");
  const replacement = persisted("regex.replace", "");
  const init = untrack(() => input);
  if (init !== undefined) text.value = init;

  const FLAGS: [string, string][] = [
    ["g", "全局"],
    ["i", "忽略大小写"],
    ["m", "多行"],
    ["s", ". 匹配换行"],
    ["u", "Unicode"],
  ];

  /** 常用正则，点一下填进去 */
  const PRESETS: [string, string][] = [
    ["手机号", "1[3-9]\\d{9}"],
    ["邮箱", "[\\w.+-]+@[\\w-]+(\\.[\\w-]+)+"],
    ["网址", "https?://[^\\s\"'<>]+"],
    ["IPv4", "\\b(?:(?:25[0-5]|2[0-4]\\d|1?\\d?\\d)\\.){3}(?:25[0-5]|2[0-4]\\d|1?\\d?\\d)\\b"],
    ["身份证号", "[1-9]\\d{5}(?:19|20)\\d{2}(?:0[1-9]|1[0-2])(?:0[1-9]|[12]\\d|3[01])\\d{3}[\\dXx]"],
    ["日期", "\\d{4}-\\d{2}-\\d{2}"],
    ["中文", "[\\u4e00-\\u9fa5]+"],
    ["十六进制颜色", "#(?:[0-9a-fA-F]{3}){1,2}\\b"],
  ];

  function toggleFlag(f: string) {
    flags.value = flags.value.includes(f) ? flags.value.replace(f, "") : flags.value + f;
  }

  const compiled = $derived.by((): { re?: RegExp; error?: string } => {
    if (!pattern.value) return {};
    try {
      return { re: new RegExp(pattern.value, flags.value) };
    } catch (e) {
      return { error: (e as Error).message };
    }
  });

  const matches = $derived.by(() => {
    const re = compiled.re;
    if (!re) return [];
    const g = new RegExp(re.source, re.flags.includes("g") ? re.flags : re.flags + "g");
    const out: RegExpExecArray[] = [];
    for (const m of text.value.matchAll(g)) {
      out.push(m as RegExpExecArray);
      if (out.length >= 5000 || !re.flags.includes("g")) break;
    }
    return out;
  });

  /** 把文本按匹配拆成片段，匹配到的高亮 */
  const segments = $derived.by(() => {
    const segs: { text: string; hit: boolean }[] = [];
    let pos = 0;
    for (const m of matches) {
      if (m.index > pos) segs.push({ text: text.value.slice(pos, m.index), hit: false });
      if (m[0]) segs.push({ text: m[0], hit: true });
      pos = m.index + m[0].length;
    }
    segs.push({ text: text.value.slice(pos), hit: false });
    return segs;
  });

  let hl = $state<HTMLDivElement>();

  const replaced = $derived(compiled.re && replacement.value ? text.value.replace(compiled.re, replacement.value) : "");
</script>

<div class="tool">
  <div class="pattern">
    <span class="slash mono">/</span>
    <input class="mono grow" spellcheck="false" bind:value={pattern.value} placeholder={t("正则表达式")} />
    <span class="slash mono">/{flags.value}</span>
    {#each FLAGS as [f, name] (f)}
      <button class="flag" class:on={flags.value.includes(f)} title={t(name)} onclick={() => toggleFlag(f)}>{f}</button>
    {/each}
  </div>
  <div class="presets">
    {#each PRESETS as [name, re] (name)}
      <button class="ghost" onclick={() => (pattern.value = re)}>{t(name)}</button>
    {/each}
  </div>
  {#if compiled.error}<div class="tool-error">{compiled.error}</div>{/if}

  <div class="tool-cols">
    <Pane label={t("测试文本")}>
      <div class="editor">
        <div class="highlight mono" aria-hidden="true" bind:this={hl}>{#each segments as s, i (i)}{#if s.hit}<mark>{s.text}</mark>{:else}{s.text}{/if}{/each}{"\n"}</div>
        <textarea class="mono" spellcheck="false" bind:value={text.value} onscroll={(e) => hl && (hl.scrollTop = e.currentTarget.scrollTop)}></textarea>
      </div>
    </Pane>
    <Pane label={t("匹配结果（{0}）", matches.length)}>
      <div class="matches">
        {#each matches.slice(0, 500) as m, i (i)}
          <div class="match">
            <div><span class="faint">#{i + 1} · {m.index}</span> <span class="mono">{m[0]}</span></div>
            {#each m.slice(1) as g, gi (gi)}
              <div class="group mono"><span class="faint">${gi + 1}</span> {g ?? "undefined"}</div>
            {/each}
            {#if m.groups}
              {#each Object.entries(m.groups) as [k, v] (k)}
                <div class="group mono"><span class="faint">{k}</span> {v}</div>
              {/each}
            {/if}
          </div>
        {/each}
      </div>
    </Pane>
  </div>
  <div class="tool-bar">
    <input class="mono grow" bind:value={replacement.value} placeholder={t("替换为（可以用 $1、$<name>）")} />
  </div>
  {#if replaced}
    <Pane label={t("替换结果")} grow={false}>
      {#snippet actions()}<CopyButton text={replaced} />{/snippet}
      <textarea class="code" rows="4" readonly value={replaced}></textarea>
    </Pane>
  {/if}
</div>

<style>
  .pattern {
    display: flex;
    align-items: center;
    gap: 6px;
  }
  .slash {
    color: var(--text-faint);
    font-size: 15px;
  }
  .grow {
    flex: 1;
  }
  .flag {
    width: 28px;
    padding: 0;
    font-family: var(--mono);
  }
  .flag.on {
    background: var(--accent);
    border-color: var(--accent);
    color: #fff;
  }
  .presets {
    display: flex;
    flex-wrap: wrap;
    gap: 4px;
    margin-top: -4px;
  }
  .presets button {
    height: 24px;
    padding: 0 8px;
    color: var(--text-dim);
    font-size: 12px;
  }
  /* 高亮层和输入框叠在一起，字体、内边距必须完全一致 */
  .editor {
    position: relative;
    flex: 1;
    min-height: 0;
  }
  .editor textarea,
  .highlight {
    position: absolute;
    inset: 0;
    margin: 0;
    padding: 7px 9px;
    border: 1px solid transparent;
    font-size: 12.5px;
    line-height: 1.45;
    white-space: pre-wrap;
    word-break: break-word;
    overflow: auto;
  }
  .editor textarea {
    width: 100%;
    height: 100%;
    background: transparent;
    border-color: var(--border);
    resize: none;
    caret-color: var(--text);
  }
  .highlight {
    color: transparent;
    background: var(--input-bg);
    border-radius: 5px;
    pointer-events: none;
  }
  mark {
    color: transparent;
    background: var(--bg-hit);
    border-radius: 2px;
    box-shadow: 0 0 0 1px var(--warn);
  }
  .matches {
    flex: 1;
    overflow: auto;
    border: 1px solid var(--border);
    border-radius: 5px;
    background: var(--bg-card);
    user-select: text;
    -webkit-user-select: text;
  }
  .match {
    padding: 6px 10px;
    border-bottom: 1px solid var(--border);
  }
  .group {
    padding-left: 14px;
    color: var(--text-dim);
  }
</style>
