<script lang="ts">
  import type { Clip } from "../lib/api";
  import { clipImageUrl } from "../lib/images";

  let { clip, full = false }: { clip: Clip; full?: boolean } = $props();

  let url = $state("");
  $effect(() => {
    let stale = false;
    clipImageUrl(clip.id, !full).then((u) => !stale && (url = u)).catch(() => {});
    return () => (stale = true);
  });
</script>

<div class="thumb" class:full>
  {#if url}<img src={url} alt="" draggable="false" />{/if}
</div>

<style>
  .thumb {
    display: flex;
    align-items: center;
    height: 64px;
  }
  .thumb.full {
    justify-content: center;
    height: 100%;
  }
  img {
    max-width: 100%;
    max-height: 100%;
    border-radius: 4px;
    object-fit: contain;
    background: repeating-conic-gradient(var(--bg-empty) 0 25%, transparent 0 50%) 0 0 / 12px 12px;
  }
</style>
