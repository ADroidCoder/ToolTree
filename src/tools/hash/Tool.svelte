<script lang="ts">
  import { untrack } from "svelte";
  import { open } from "@tauri-apps/plugin-dialog";
  import { FileUp } from "@lucide/svelte";
  import Pane from "../../components/Pane.svelte";
  import Seg from "../../components/Seg.svelte";
  import ResultRow from "../../components/ResultRow.svelte";
  import { api, type Hashes } from "../../lib/api";
  import { t } from "../../lib/i18n.svelte";
  import { persisted } from "../../lib/persist.svelte";
  import { bytesToBase64, formatSize, utf8 } from "../../lib/util";
  import type { ToolProps } from "../types";

  let { input }: ToolProps = $props();

  const tab = persisted<"text" | "file" | "hmac">("hash.tab", "text");
  const text = persisted("hash.input", "");
  const upper = persisted("hash.upper", false);
  const init = untrack(() => input);
  if (init !== undefined) {
    text.value = init;
    tab.value = "text";
  }

  let hashes = $state<Hashes | null>(null);
  let file = $state("");
  let busy = $state(false);
  let error = $state("");
  /** 粘贴一个期望的哈希值，和结果比对 */
  let expect = $state("");

  $effect(() => {
    if (tab.value !== "text") return;
    const s = text.value;
    let stale = false;
    api.hashText(s).then((h) => !stale && (hashes = h)).catch((e) => (error = String(e)));
    return () => (stale = true);
  });

  async function pickFile() {
    const p = await open({ multiple: false, directory: false });
    if (typeof p !== "string") return;
    file = p;
    hashes = null;
    error = "";
    busy = true;
    try {
      hashes = await api.hashFile(p);
    } catch (e) {
      error = String(e);
    } finally {
      busy = false;
    }
  }

  // HMAC 用浏览器自带的 WebCrypto
  let hmacKey = $state("");
  let hmac = $state<Record<string, string>>({});
  const hmacEncoding = persisted<"hex" | "base64">("hash.hmacEnc", "hex");
  $effect(() => {
    if (tab.value !== "hmac") return;
    const msg = text.value;
    const key = hmacKey;
    const enc = hmacEncoding.value;
    let stale = false;
    (async () => {
      const out: Record<string, string> = {};
      for (const alg of ["SHA-1", "SHA-256", "SHA-384", "SHA-512"]) {
        if (!key) break;
        const k = await crypto.subtle.importKey("raw", utf8.encode(key), { name: "HMAC", hash: alg }, false, ["sign"]);
        const sig = new Uint8Array(await crypto.subtle.sign("HMAC", k, utf8.encode(msg)));
        out[alg] = enc === "hex" ? [...sig].map((b) => b.toString(16).padStart(2, "0")).join("") : bytesToBase64(sig);
      }
      if (!stale) hmac = out;
    })();
    return () => (stale = true);
  });

  const shown = $derived(tab.value === "file" && !file ? null : hashes);
  const fmt = (s: string) => (upper.value ? s.toUpperCase() : s);
  const match = (s: string) => expect.trim() && s.toLowerCase() === expect.trim().toLowerCase();
</script>

<div class="tool">
  <div class="tool-bar">
    <Seg bind:value={tab.value} options={[{ value: "text", label: t("文本") }, { value: "file", label: t("文件") }, { value: "hmac", label: "HMAC" }]} />
    <label class="check"><input type="checkbox" bind:checked={upper.value} />{t("大写")}</label>
  </div>

  {#if tab.value === "file"}
    <div class="file">
      <button class="icon" onclick={pickFile} disabled={busy}><FileUp size={14} />{t("选择文件")}</button>
      <span class="dim mono path">{busy ? t("正在计算…") : file}</span>
      {#if hashes && file && !busy}<span class="faint">{formatSize(hashes.size)}</span>{/if}
    </div>
  {:else}
    <Pane label={t("文本")} grow={false}>
      <textarea class="code" rows="6" spellcheck="false" bind:value={text.value}></textarea>
    </Pane>
  {/if}

  {#if error}<div class="tool-error">{error}</div>{/if}

  {#if tab.value === "hmac"}
    <div class="tool-bar">
      <input class="mono grow" bind:value={hmacKey} placeholder={t("密钥")} />
      <Seg bind:value={hmacEncoding.value} options={[{ value: "hex", label: "Hex" }, { value: "base64", label: "Base64" }]} />
    </div>
    <div class="tool-rows">
      {#each Object.entries(hmac) as [alg, v] (alg)}
        <ResultRow label={`HMAC-${alg}`} value={hmacEncoding.value === "hex" ? fmt(v) : v} />
      {/each}
    </div>
  {:else if shown}
    <div class="tool-rows">
      {#each [["MD5", shown.md5], ["SHA1", shown.sha1], ["SHA256", shown.sha256], ["SHA384", shown.sha384], ["SHA512", shown.sha512]] as [name, v] (name)}
        <div class:match={match(v)}>
          <ResultRow label={name} value={fmt(v)} />
        </div>
      {/each}
    </div>
    <input class="mono" bind:value={expect} placeholder={t("粘贴期望的哈希值，比对是否一致")} />
    {#if expect.trim()}
      {@const ok = [shown.md5, shown.sha1, shown.sha256, shown.sha384, shown.sha512].some(match)}
      <div class={ok ? "ok" : "tool-error"}>{ok ? t("一致") : t("和以上任何一个都不一致")}</div>
    {/if}
  {/if}
</div>

<style>
  .file {
    display: flex;
    align-items: center;
    gap: 12px;
    min-width: 0;
  }
  .path {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .grow {
    flex: 1;
  }
  .match :global(.row) {
    border-color: var(--add);
    background: var(--bg-add);
  }
  .ok {
    padding: 6px 10px;
    border-radius: 5px;
    background: var(--bg-add);
    color: var(--add);
    font-size: 12px;
  }
</style>
