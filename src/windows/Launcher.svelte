<!-- 全局快捷键弹出的启动器：一个搜索框搜应用、工具、文件，输入算式直接出结果，粘贴内容自动识别。
     回车打开，⌘回车在访达里显示，Esc 先清空再关闭 -->
<script lang="ts">
  import { onMount, type Component } from "svelte";
  import { listen } from "@tauri-apps/api/event";
  import { homeDir } from "@tauri-apps/api/path";
  import { AppWindow, Calculator, ClipboardList, File, Folder, House, Scissors, Search, Settings } from "@lucide/svelte";
  import { api, type AppEntry, type FileHit } from "../lib/api";
  import { calc, groupDigits } from "../lib/calc";
  import { detect } from "../lib/detect";
  import { i18n, t, watchLang } from "../lib/i18n.svelte";
  import { forget, iconUrl, markUsed, recent, shortPath, usageBoost, type Used } from "../lib/launcher";
  import { best, keys, type Keys } from "../lib/match";
  import { applyTheme, watchSystemTheme } from "../lib/settings.svelte";
  import { isMac, modKey } from "../lib/util";
  import { TOOLS, findTool } from "../tools/registry";

  type Group = "calc" | "detect" | "recent" | "app" | "tool" | "command" | "file";

  interface Item {
    key: string;
    group: Group;
    title: string;
    sub?: string;
    icon: Component;
    /** 有路径的条目（应用、文件）显示系统图标 */
    path?: string;
    score: number;
    run: () => void;
    /** 在访达 / 资源管理器里显示 */
    reveal?: () => void;
  }

  const GROUP_NAMES: Record<Group, string> = {
    calc: "计算",
    detect: "识别到的内容",
    recent: "最近使用",
    app: "应用",
    tool: "工具",
    command: "命令",
    file: "文件",
  };
  /** 每组最多显示几条 */
  const LIMIT: Record<Group, number> = { calc: 1, detect: 3, recent: 8, app: 6, tool: 5, command: 3, file: 8 };
  /** 以这些开头时只搜文件，显示更多 */
  const FILE_PREFIX = /^(f|find|找|文件)\s+/i;

  let q = $state("");
  let index = $state(0);
  let input = $state<HTMLInputElement>();
  let listEl = $state<HTMLDivElement>();
  let apps = $state.raw<{ app: AppEntry; keys: Keys[] }[]>([]);
  let files = $state.raw<FileHit[]>([]);
  /** files 是哪次搜索的结果，和当前输入不一致时不显示 */
  let filesFor = $state("");
  let home = $state("");

  // ---- 数据 ----

  async function loadApps() {
    const list = await api.launcherApps().catch(() => []);
    apps = list.map((app) => ({ app, keys: [keys(app.name), ...(app.alt && app.alt !== app.name ? [keys(app.alt)] : [])] }));
  }

  const toolKeys = $derived.by(() => {
    void i18n.lang;
    return TOOLS.map((tool) => ({ tool, keys: [tool.name, t(tool.name), tool.id, ...tool.keywords].map(keys) }));
  });

  function hideThen(fn: () => void) {
    api.launcherHide().finally(fn);
  }

  const COMMANDS: { id: string; name: string; icon: Component; words: string[]; run: () => void }[] = [
    {
      id: "shot",
      name: "截图",
      icon: Scissors,
      words: ["screenshot", "jietu", "jt", "capture"],
      // 等启动器从屏幕上消失再截
      run: () => hideThen(() => setTimeout(() => api.shotStart(false), 120)),
    },
    { id: "clipboard", name: "剪贴板历史", icon: ClipboardList, words: ["clipboard", "jianqieban", "jqb"], run: () => api.showMain("clipboard") },
    { id: "home", name: "打开 ToolTree", icon: House, words: ["tooltree", "main"], run: () => api.showMain("home") },
    { id: "settings", name: "ToolTree 设置", icon: Settings, words: ["settings", "preferences", "shezhi", "sz"], run: () => api.showMain("settings") },
  ];
  const commandKeys = $derived.by(() => {
    void i18n.lang;
    return COMMANDS.map((c) => ({ c, keys: [c.name, t(c.name), ...c.words].map(keys) }));
  });

  // 文件：停顿一下再搜，打字时不用每个字搜一次
  const fileMode = $derived(FILE_PREFIX.test(q));
  const fileQuery = $derived(q.replace(FILE_PREFIX, "").trim());
  let fileSeq = 0;
  $effect(() => {
    const fq = fileQuery;
    const limit = fileMode ? 40 : LIMIT.file;
    const my = ++fileSeq;
    // 一个汉字就能搜，英文至少两个字母
    if (fq.length < 2 && !/[㐀-鿿]/.test(fq)) {
      files = [];
      filesFor = "";
      return;
    }
    const timer = setTimeout(async () => {
      const hits = await api.launcherFiles(fq, limit).catch(() => []);
      if (my !== fileSeq) return;
      files = hits;
      filesFor = fq;
    }, 140);
    return () => clearTimeout(timer);
  });

  // ---- 条目 ----

  function appItem(a: AppEntry, group: Group, score: number): Item {
    return {
      key: "app:" + a.path,
      group,
      title: a.name,
      sub: a.alt && a.alt !== a.name ? a.alt : undefined,
      icon: AppWindow,
      path: a.path,
      score,
      run: () => {
        markUsed("app", a.path, a.name);
        api.launcherOpen(a.path);
      },
      reveal: () => api.launcherOpen(a.path, true),
    };
  }

  function fileItem(f: { path: string; name: string; dir: boolean }, group: Group, score: number): Item {
    const parent = f.path.slice(0, Math.max(f.path.lastIndexOf("/"), f.path.lastIndexOf("\\")));
    return {
      key: "file:" + f.path,
      group,
      title: f.name,
      sub: shortPath(parent, home),
      icon: f.dir ? Folder : File,
      path: f.path,
      score,
      run: () => {
        markUsed("file", f.path, f.name);
        api.launcherOpen(f.path);
      },
      reveal: () => api.launcherOpen(f.path, true),
    };
  }

  function toolItem(id: string, group: Group, score: number, input?: string): Item | null {
    const tool = findTool(id);
    if (!tool) return null;
    return {
      key: "tool:" + id + (input ? ":in" : ""),
      group,
      title: t(tool.name),
      sub: t(tool.desc),
      icon: tool.icon,
      score,
      run: () => {
        markUsed("tool", id, tool.name);
        api.showMain(id, input);
      },
    };
  }

  function commandItem(id: string, group: Group, score: number): Item | null {
    const c = COMMANDS.find((x) => x.id === id);
    if (!c) return null;
    return {
      key: "cmd:" + id,
      group,
      title: t(c.name),
      icon: c.icon,
      score,
      run: () => {
        markUsed("command", id, c.name);
        c.run();
      },
    };
  }

  function recentItem(u: Used): Item | null {
    if (u.kind === "app") {
      const a = apps.find((x) => x.app.path === u.id)?.app;
      // 应用被删掉了就不再显示
      if (!a) {
        if (apps.length) forget("app", u.id);
        return null;
      }
      return appItem(a, "recent", 0);
    }
    if (u.kind === "file") return fileItem({ path: u.id, name: u.title, dir: false }, "recent", 0);
    if (u.kind === "tool") return toolItem(u.id, "recent", 0);
    return commandItem(u.id, "recent", 0);
  }

  const items = $derived.by((): Item[] => {
    const s = q.trim();
    if (!s) return recent(LIMIT.recent).map(recentItem).filter((x) => x !== null);
    const out: Item[] = [];

    if (!fileMode) {
      const r = calc(s);
      if (r) {
        out.push({
          key: "calc",
          group: "calc",
          title: "= " + groupDigits(r.text),
          sub: t("回车复制结果"),
          icon: Calculator,
          score: 1000,
          run: () => api.copyText(r.text).then(() => api.launcherHide()),
        });
      }
      for (const d of detect(s).slice(0, LIMIT.detect)) {
        const it = toolItem(d.tool, "detect", 900, s);
        if (it) out.push({ ...it, key: "detect:" + d.tool + d.label, title: `${t(d.label)} → ${it.title}`, sub: d.preview });
      }

      // 应用、工具、命令按各自最高分决定哪组排前面
      const groups: Item[][] = [];
      const pick = (list: Item[], n: number) => list.sort((a, b) => b.score - a.score).slice(0, n);
      groups.push(
        pick(
          apps.flatMap(({ app, keys }) => {
            const sc = best(s, keys);
            return sc ? [appItem(app, "app", sc + usageBoost("app", app.path))] : [];
          }),
          LIMIT.app,
        ),
      );
      groups.push(
        pick(
          toolKeys.flatMap(({ tool, keys }) => {
            const sc = best(s, keys);
            const it = sc ? toolItem(tool.id, "tool", sc + usageBoost("tool", tool.id)) : null;
            return it ? [it] : [];
          }),
          LIMIT.tool,
        ),
      );
      groups.push(
        pick(
          commandKeys.flatMap(({ c, keys }) => {
            const sc = best(s, keys);
            const it = sc ? commandItem(c.id, "command", sc + usageBoost("command", c.id)) : null;
            return it ? [it] : [];
          }),
          LIMIT.command,
        ),
      );
      groups.sort((a, b) => (b[0]?.score ?? 0) - (a[0]?.score ?? 0));
      for (const g of groups) out.push(...g);
    }

    if (filesFor && filesFor === fileQuery) out.push(...files.map((f) => fileItem(f, "file", 0)));
    return out;
  });

  /** 每组第一条前面显示组名 */
  const firstOfGroup = $derived(new Set(items.filter((it, i) => i === 0 || items[i - 1].group !== it.group).map((it) => it.key)));
  const selected = $derived(items[index]);

  $effect(() => {
    void q;
    index = 0;
  });

  $effect(() => {
    if (index >= items.length) index = Math.max(0, items.length - 1);
  });

  // ---- 交互 ----

  function onShow() {
    q = "";
    index = 0;
    files = [];
    filesFor = "";
    loadApps();
    // 窗口刚显示时还没拿到焦点，等一下再聚焦
    requestAnimationFrame(() => input?.focus());
  }

  onMount(() => {
    applyTheme();
    const stopTheme = watchSystemTheme();
    const stopLang = watchLang();
    homeDir().then((h) => (home = h)).catch(() => {});
    const un = listen("launcher-show", onShow);
    onShow();
    return () => {
      stopTheme();
      stopLang();
      un.then((f) => f());
    };
  });

  function run(it: Item | undefined, reveal = false) {
    if (!it) return;
    if (reveal) {
      if (it.reveal) it.reveal();
      return;
    }
    it.run();
  }

  function move(d: number) {
    if (!items.length) return;
    index = (index + d + items.length) % items.length;
    requestAnimationFrame(() => listEl?.querySelector(`[data-i="${index}"]`)?.scrollIntoView({ block: "nearest" }));
  }

  function onKey(e: KeyboardEvent) {
    // 输入法还在选字时的回车、方向键是给输入法的
    if (e.isComposing) return;
    const ctrl = isMac && e.ctrlKey;
    if (e.key === "ArrowDown" || (ctrl && e.key === "n")) move(1);
    else if (e.key === "ArrowUp" || (ctrl && e.key === "p")) move(-1);
    else if (e.key === "Enter") run(selected, modKey(e));
    else if (e.key === "Escape") {
      if (q) q = "";
      else api.launcherHide();
    } else return;
    e.preventDefault();
  }

  const mod = isMac ? "⌘" : "Ctrl+";
</script>

<svelte:window onkeydown={onKey} />

<div class="launcher">
  <div class="bar">
    <Search size={18} />
    <input
      bind:this={input}
      bind:value={q}
      placeholder={t("搜索应用、文件和工具，或者输入算式")}
      spellcheck="false"
      autocomplete="off"
    />
  </div>

  <div class="list" bind:this={listEl}>
    {#each items as it, i (it.key)}
      {#if firstOfGroup.has(it.key)}
        <div class="group">{t(GROUP_NAMES[it.group])}</div>
      {/if}
      <button class="row" class:on={i === index} class:calc={it.group === "calc"} data-i={i} onmousemove={() => (index = i)} onclick={() => run(it)}>
        <span class="icon">
          {#if it.path}
            {#await iconUrl(it.path)}
              <it.icon size={18} />
            {:then url}
              {#if url}<img src={url} alt="" />{:else}<it.icon size={18} />{/if}
            {/await}
          {:else}
            <it.icon size={18} />
          {/if}
        </span>
        <span class="title">{it.title}</span>
        {#if it.sub}<span class="sub">{it.sub}</span>{/if}
      </button>
    {:else}
      <div class="empty">
        {#if q.trim()}
          {t("没有找到")}
        {:else}
          {t("输入应用名、文件名、工具名，或者 12*8+3 这样的算式")}
          <small>{t("以“f ”开头只搜文件")}</small>
        {/if}
      </div>
    {/each}
  </div>

  <div class="foot">
    {#if selected}
      <span><kbd>↵</kbd> {selected.group === "calc" ? t("复制") : t("打开")}</span>
      {#if selected.reveal}<span><kbd>{mod}↵</kbd> {isMac ? t("在访达中显示") : t("打开所在文件夹")}</span>{/if}
    {/if}
    <span class="right"><kbd>esc</kbd> {q ? t("清空") : t("关闭")}</span>
  </div>
</div>

<style>
  .launcher {
    display: flex;
    flex-direction: column;
    height: 100vh;
    background: var(--bg-graph);
    border: 1px solid var(--border);
    border-radius: 12px;
    overflow: hidden;
  }
  .bar {
    flex: 0 0 56px;
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 0 18px;
    border-bottom: 1px solid var(--border);
    color: var(--text-faint);
  }
  .bar input {
    flex: 1;
    height: 100% !important;
    padding: 0 !important;
    border: 0 !important;
    background: transparent !important;
    color: var(--text);
    font-size: 18px !important;
    box-shadow: none !important;
  }
  .list {
    flex: 1;
    min-height: 0;
    overflow-y: auto;
    padding: 4px 8px 8px;
  }
  .group {
    padding: 10px 10px 4px;
    color: var(--text-faint);
    font-size: 11px;
    font-weight: 600;
  }
  .row {
    display: flex;
    align-items: center;
    gap: 10px;
    width: 100%;
    height: 40px;
    padding: 0 10px;
    border: 0;
    border-radius: 8px;
    background: transparent;
    color: var(--text-dim);
    text-align: left;
  }
  .row.on {
    background: var(--bg-selected);
    color: var(--text);
  }
  .row.calc {
    height: 52px;
  }
  .row.calc .title {
    font-size: 22px;
    font-weight: 600;
    font-variant-numeric: tabular-nums;
  }
  .icon {
    flex: 0 0 28px;
    display: flex;
    align-items: center;
    justify-content: center;
    height: 28px;
  }
  .icon img {
    width: 28px;
    height: 28px;
  }
  .title {
    color: var(--text);
    font-size: 14px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    max-width: 60%;
  }
  .sub {
    flex: 1;
    min-width: 0;
    color: var(--text-faint);
    font-size: 12px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .empty {
    display: flex;
    flex-direction: column;
    gap: 6px;
    padding: 48px 20px;
    color: var(--text-faint);
    text-align: center;
  }
  .empty small {
    font-size: 11.5px;
  }
  .foot {
    flex: 0 0 30px;
    display: flex;
    align-items: center;
    gap: 14px;
    padding: 0 14px;
    border-top: 1px solid var(--border);
    background: var(--bg-side);
    color: var(--text-faint);
    font-size: 11px;
  }
  .foot .right {
    margin-left: auto;
  }
</style>
