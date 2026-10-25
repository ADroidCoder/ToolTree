// JSON 的解析、出错位置、转换

/** 解析失败时算出第几行第几列，界面上好定位 */
export function parseJson(text: string): { value?: unknown; error?: string } {
  try {
    return { value: JSON.parse(text) };
  } catch (e) {
    const msg = (e as Error).message;
    const m = /position (\d+)/.exec(msg);
    if (!m) return { error: msg };
    const pos = Number(m[1]);
    const before = text.slice(0, pos).split("\n");
    return { error: `${msg}（${before.length}:${before[before.length - 1].length + 1}）` };
  }
}

export function sortKeys(v: unknown): unknown {
  if (Array.isArray(v)) return v.map(sortKeys);
  if (v && typeof v === "object") {
    return Object.fromEntries(
      Object.keys(v)
        .sort()
        .map((k) => [k, sortKeys((v as Record<string, unknown>)[k])]),
    );
  }
  return v;
}

const IDENT = /^[A-Za-z_$][\w$]*$/;

function pascal(s: string) {
  const r = s.replace(/[^A-Za-z0-9]+(.)?/g, (_, c: string | undefined) => (c ? c.toUpperCase() : "")).replace(/^./, (c) => c.toUpperCase());
  return /^[A-Za-z_]/.test(r) ? r : "T" + r;
}

/** 根据一份 JSON 示例生成 TypeScript 类型。数组里的对象合并成一个类型，不是每个元素都有的字段标成可选 */
export function toTypeScript(value: unknown, rootName = "Root"): string {
  const out: string[] = [];
  const used = new Set<string>();

  function name(base: string) {
    let n = pascal(base) || "Item";
    let i = 2;
    while (used.has(n)) n = pascal(base) + i++;
    used.add(n);
    return n;
  }

  function merge(objs: Record<string, unknown>[]) {
    const keys = new Map<string, unknown[]>();
    for (const o of objs) for (const [k, v] of Object.entries(o)) keys.set(k, [...(keys.get(k) ?? []), v]);
    return { keys, count: objs.length };
  }

  function typeOf(values: unknown[], hint: string): string {
    const types = new Set<string>();
    const objs: Record<string, unknown>[] = [];
    const arrays: unknown[] = [];
    for (const v of values) {
      if (v === null) types.add("null");
      else if (Array.isArray(v)) arrays.push(...v), types.add("__array");
      else if (typeof v === "object") objs.push(v as Record<string, unknown>);
      else types.add(typeof v);
    }
    if (objs.length) types.add(iface(objs, hint));
    if (types.has("__array")) {
      types.delete("__array");
      const inner = arrays.length ? typeOf(arrays, hint.replace(/s$/, "") + (hint.endsWith("s") ? "" : "Item")) : "unknown";
      types.add(inner.includes(" | ") ? `(${inner})[]` : `${inner}[]`);
    }
    return [...types].join(" | ") || "unknown";
  }

  function iface(objs: Record<string, unknown>[], hint: string): string {
    const n = name(hint);
    const { keys, count } = merge(objs);
    const lines = [...keys].map(([k, vs]) => {
      const key = IDENT.test(k) ? k : JSON.stringify(k);
      const opt = vs.length < count ? "?" : "";
      return `  ${key}${opt}: ${typeOf(vs, k)};`;
    });
    out.push(`export interface ${n} {\n${lines.join("\n")}\n}`);
    return n;
  }

  const root = typeOf([value], rootName);
  if (!out.length || root !== pascal(rootName)) out.push(`export type ${pascal(rootName)}Type = ${root};`);
  return out.reverse().join("\n\n");
}
