<script lang="ts">
  import { untrack } from "svelte";
  import Pane from "../../components/Pane.svelte";
  import CopyButton from "../../components/CopyButton.svelte";
  import ResultRow from "../../components/ResultRow.svelte";
  import { t } from "../../lib/i18n.svelte";
  import { persisted } from "../../lib/persist.svelte";
  import { formatDateTime } from "../../lib/util";
  import { canVerify, decodeJwt, verifyHmac } from "./jwt";
  import type { ToolProps } from "../types";

  let { input }: ToolProps = $props();

  const token = persisted("jwt.input", "");
  const init = untrack(() => input);
  if (init !== undefined) token.value = init;
  let secret = $state("");
  let verified = $state<boolean | null>(null);

  const decoded = $derived(token.value.trim() ? decodeJwt(token.value) : {});
  const header = $derived(decoded.value ? JSON.stringify(decoded.value.header, null, 2) : "");
  const payload = $derived(
    decoded.value ? (typeof decoded.value.payload === "string" ? decoded.value.payload : JSON.stringify(decoded.value.payload, null, 2)) : "",
  );
  const claims = $derived((decoded.value?.payload ?? {}) as Record<string, unknown>);
  const alg = $derived(decoded.value?.header.alg);

  const TIME_CLAIMS: [string, string][] = [
    ["iat", "签发时间"],
    ["nbf", "生效时间"],
    ["exp", "过期时间"],
  ];

  $effect(() => {
    const tk = token.value;
    const s = secret;
    const a = alg;
    verified = null;
    if (!s || !canVerify(a)) return;
    let stale = false;
    verifyHmac(tk, a as string, s).then((ok) => !stale && (verified = ok)).catch(() => {});
    return () => (stale = true);
  });

  const expired = $derived(typeof claims.exp === "number" ? claims.exp * 1000 < Date.now() : null);
</script>

<div class="tool">
  <Pane label="Token" grow={false}>
    {#snippet actions()}
      <button onclick={() => (token.value = "")}>{t("清空")}</button>
    {/snippet}
    <textarea class="code wrap" rows="4" spellcheck="false" bind:value={token.value} placeholder="eyJhbGciOi..."></textarea>
  </Pane>

  {#if decoded.error}
    <div class="tool-error">{t(decoded.error)}</div>
  {:else if decoded.value}
    <div class="status">
      <span class="tag">{t("算法")}：{String(alg ?? "—")}</span>
      {#if expired !== null}
        <span class="tag" class:bad={expired} class:good={!expired}>{expired ? t("已过期") : t("未过期")}</span>
      {/if}
      {#if verified !== null}
        <span class="tag" class:bad={!verified} class:good={verified}>{verified ? t("签名正确") : t("签名不匹配")}</span>
      {/if}
    </div>
    {#if TIME_CLAIMS.some(([k]) => typeof claims[k] === "number")}
      <div class="tool-rows">
        {#each TIME_CLAIMS as [k, name] (k)}
          {#if typeof claims[k] === "number"}
            <ResultRow label={`${k} · ${t(name)}`} value={formatDateTime((claims[k] as number) * 1000)} />
          {/if}
        {/each}
      </div>
    {/if}
    <div class="tool-cols">
      <Pane label={t("头部")}>
        {#snippet actions()}<CopyButton text={header} />{/snippet}
        <textarea class="code" readonly value={header}></textarea>
      </Pane>
      <Pane label={t("载荷")}>
        {#snippet actions()}<CopyButton text={payload} />{/snippet}
        <textarea class="code" readonly value={payload}></textarea>
      </Pane>
    </div>
    {#if canVerify(alg)}
      <div class="tool-bar">
        <span class="dim">{t("校验签名")}</span>
        <input class="grow mono" type="text" bind:value={secret} placeholder={t("输入 {0} 密钥", String(alg))} />
      </div>
    {/if}
  {/if}
</div>

<style>
  .wrap {
    white-space: pre-wrap;
    word-break: break-all;
  }
  .status {
    display: flex;
    gap: 8px;
  }
  .tag {
    padding: 3px 9px;
    border-radius: 10px;
    background: var(--bg-card);
    border: 1px solid var(--border);
    font-size: 12px;
  }
  .tag.good {
    color: var(--add);
    background: var(--bg-add);
    border-color: transparent;
  }
  .tag.bad {
    color: var(--del);
    background: var(--bg-del);
    border-color: transparent;
  }
  .grow {
    flex: 1;
  }
</style>
