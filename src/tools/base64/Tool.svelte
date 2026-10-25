<script lang="ts">
  import { untrack } from "svelte";
  import Pane from "../../components/Pane.svelte";
  import Seg from "../../components/Seg.svelte";
  import CopyButton from "../../components/CopyButton.svelte";
  import { t } from "../../lib/i18n.svelte";
  import { persisted } from "../../lib/persist.svelte";
  import { copy } from "../../lib/store.svelte";
  import { base64ToBytes, bytesToBase64, formatSize, utf8 } from "../../lib/util";
  import type { ToolProps } from "../types";

  let { input }: ToolProps = $props();

  const tab = persisted<"text" | "file">("base64.tab", "text");
  const mode = persisted<"encode" | "decode">("base64.mode", "encode");
  const urlSafe = persisted("base64.urlSafe", false);
  const text = persisted("base64.input", "");
  const init = untrack(() => input);
  if (init !== undefined) {
    text.value = init;
    mode.value = "decode";
    tab.value = "text";
  }

  const result = $derived.by((): { out: string; error?: string } => {
    const s = text.value;
    if (!s) return { out: "" };
    try {
      if (mode.value === "encode") {
        let b = bytesToBase64(utf8.encode(s));
        if (urlSafe.value) b = b.replace(/\+/g, "-").replace(/\//g, "_").replace(/=+$/, "");
        return { out: b };
      }
      // data:image/png;base64,xxx 这种也能直接解
      const body = s.trim().replace(/^data:[^,]*;base64,/, "");
      return { out: utf8.decode(base64ToBytes(body), true) };
    } catch {
      return { out: "", error: mode.value === "decode" ? t("不是有效的 Base64，或者解码后不是 UTF-8 文本") : t("编码失败") };
    }
  });

  function swap() {
    if (result.error) return;
    text.value = result.out;
    mode.value = mode.value === "encode" ? "decode" : "encode";
  }

  // 文件和图片
  let dataUrl = $state("");
  let fileInfo = $state("");
  let pasted = $state("");
  const preview = $derived(pasted.trim().startsWith("data:image/") ? pasted.trim() : /^[A-Za-z0-9+/=\s]{40,}$/.test(pasted) ? guessImage(pasted) : "");

  /** 根据开头几个字节猜图片格式，拼成 data URL 用来预览 */
  function guessImage(b64: string) {
    const head = b64.replace(/\s/g, "").slice(0, 16);
    const type = head.startsWith("iVBOR") ? "png" : head.startsWith("/9j/") ? "jpeg" : head.startsWith("R0lG") ? "gif" : head.startsWith("UklG") ? "webp" : head.startsWith("PHN2") ? "svg+xml" : "";
    return type ? `data:image/${type};base64,${b64.replace(/\s/g, "")}` : "";
  }

  function readFile(file: File | undefined) {
    if (!file) return;
    const reader = new FileReader();
    reader.onload = () => {
      dataUrl = String(reader.result);
      fileInfo = `${file.name} · ${formatSize(file.size)} · ${file.type || "application/octet-stream"}`;
    };
    reader.readAsDataURL(file);
  }

  function onDrop(e: DragEvent) {
    e.preventDefault();
    readFile(e.dataTransfer?.files[0]);
  }
</script>

<div class="tool">
  <div class="tool-bar">
    <Seg bind:value={tab.value} options={[{ value: "text", label: t("文本") }, { value: "file", label: t("文件和图片") }]} />
    {#if tab.value === "text"}
      <Seg bind:value={mode.value} options={[{ value: "encode", label: t("编码") }, { value: "decode", label: t("解码") }]} />
      {#if mode.value === "encode"}
        <label class="check"><input type="checkbox" bind:checked={urlSafe.value} />{t("URL 安全（- _，去掉 =）")}</label>
      {/if}
    {/if}
  </div>

  {#if tab.value === "text"}
    <div class="tool-cols">
      <Pane label={mode.value === "encode" ? t("原文") : "Base64"}>
        {#snippet actions()}
          <button onclick={() => (text.value = "")}>{t("清空")}</button>
        {/snippet}
        <textarea class="code" spellcheck="false" bind:value={text.value}></textarea>
      </Pane>
      <Pane label={mode.value === "encode" ? "Base64" : t("原文")}>
        {#snippet actions()}
          <button onclick={swap} disabled={!result.out}>{t("作为输入")}</button>
          <CopyButton text={result.out} />
        {/snippet}
        {#if result.error}<div class="tool-error" style="margin-bottom: 6px">{result.error}</div>{/if}
        <textarea class="code wrap" spellcheck="false" readonly value={result.out}></textarea>
      </Pane>
    </div>
  {:else}
    <div class="tool-cols">
      <Pane label={t("文件转 Base64")}>
        {#snippet actions()}
          <CopyButton text={dataUrl} />
          <button onclick={() => copy(dataUrl.replace(/^data:[^,]*;base64,/, ""))} disabled={!dataUrl}>{t("只复制 Base64")}</button>
        {/snippet}
        <label class="drop" ondragover={(e) => e.preventDefault()} ondrop={onDrop}>
          <input type="file" hidden onchange={(e) => readFile(e.currentTarget.files?.[0])} />
          {#if dataUrl.startsWith("data:image/")}
            <img src={dataUrl} alt="" />
          {/if}
          <span>{fileInfo || t("点击选择文件，或拖到这里")}</span>
        </label>
        <textarea class="code wrap small" readonly value={dataUrl}></textarea>
      </Pane>
      <Pane label={t("Base64 预览图片")}>
        <textarea class="code wrap small" spellcheck="false" bind:value={pasted} placeholder={t("粘贴 data:image/... 或图片的 Base64")}></textarea>
        <div class="preview">
          {#if preview}
            <img src={preview} alt="" />
          {:else if pasted.trim()}
            <span class="faint">{t("识别不出图片格式")}</span>
          {/if}
        </div>
      </Pane>
    </div>
  {/if}
</div>

<style>
  .wrap {
    white-space: pre-wrap;
    word-break: break-all;
  }
  .small {
    flex: 0 0 120px !important;
    min-height: 120px;
  }
  .drop {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 10px;
    min-height: 140px;
    margin-bottom: 10px;
    border: 1.5px dashed var(--border-strong);
    border-radius: var(--radius);
    color: var(--text-dim);
    cursor: pointer;
    overflow: hidden;
  }
  .drop:hover {
    border-color: var(--accent);
  }
  .drop img,
  .preview img {
    max-width: 90%;
    max-height: 220px;
    object-fit: contain;
  }
  .preview {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    margin-top: 10px;
    min-height: 120px;
    border: 1px solid var(--border);
    border-radius: var(--radius);
    background: repeating-conic-gradient(var(--bg-empty) 0 25%, transparent 0 50%) 0 0 / 16px 16px;
  }
</style>
