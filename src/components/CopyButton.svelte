<script lang="ts">
  import { Copy } from "@lucide/svelte";
  import { copy } from "../lib/store.svelte";
  import { t } from "../lib/i18n.svelte";

  let { text, label = true }: { text: string; label?: boolean } = $props();
</script>

<button class="icon" disabled={!text} onclick={() => copy(text)} title={t("复制")}>
  <Copy size={13} />{#if label}{t("复制")}{/if}
</button>
