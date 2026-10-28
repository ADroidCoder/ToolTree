// 启动器的模糊匹配：名字前缀、单词开头、首字母缩写（vsc → Visual Studio Code）、
// 中文的全拼和拼音首字母（wx、weixin → 微信）。分数越高越靠前，0 表示不匹配。

import { pinyin } from "pinyin-pro";

/** 一个名字预先算好的各种写法，搜索时只做字符串比较 */
export interface Keys {
  name: string;
  /** 按空格、标点、大小写切开的单词 */
  words: string[];
  /** 单词首字母 */
  acronym: string;
  /** 中文转成的全拼（没有中文时为空） */
  pinyin: string;
  /** 拼音首字母，英文单词取首字母 */
  initials: string;
}

const HAN = /[㐀-鿿]/;

function splitWords(s: string) {
  return s
    .replace(/([a-z])([A-Z])/g, "$1 $2")
    .split(/[\s\-_.()（）·,，/]+/)
    .filter(Boolean)
    .map((w) => w.toLowerCase());
}

export function keys(name: string): Keys {
  const words = splitWords(name);
  const k: Keys = { name: name.toLowerCase(), words, acronym: words.map((w) => w[0]).join(""), pinyin: "", initials: "" };
  if (HAN.test(name)) {
    const full = pinyin(name, { toneType: "none", type: "array", nonZh: "consecutive", v: true });
    const first = pinyin(name, { pattern: "first", toneType: "none", type: "array", nonZh: "consecutive" });
    k.pinyin = full.join("").replace(/\s+/g, "").toLowerCase();
    // 中文取拼音首字母，夹在中间的英文单词取首字母：QQ音乐 → qyy
    k.initials = first
      .flatMap((x) => (HAN.test(x) || x.length === 1 ? [x] : splitWords(x).map((w) => w[0])))
      .join("")
      .replace(/\s+/g, "")
      .toLowerCase();
  }
  return k;
}

export function score(q: string, k: Keys): number {
  const s = q.trim().toLowerCase();
  if (!s) return 0;
  if (k.name === s) return 100;
  if (k.name.startsWith(s)) return 90;
  const compact = s.replace(/\s+/g, "");
  // 每个查询词都是某个单词的开头："studio code"、"vis co"
  const parts = s.split(/\s+/);
  if (parts.every((p) => k.words.some((w) => w.startsWith(p)))) return parts.length > 1 ? 82 : 80;
  if (compact.length >= 2 && k.acronym.startsWith(compact)) return 75;
  if (k.pinyin) {
    if (k.pinyin === compact) return 88;
    if (k.pinyin.startsWith(compact)) return 72;
    if (compact.length >= 2 && k.initials.startsWith(compact)) return 70;
  }
  if (k.name.includes(s)) return 50;
  if (k.pinyin && compact.length >= 2 && k.pinyin.includes(compact)) return 40;
  if (k.initials && compact.length >= 2 && k.initials.includes(compact)) return 35;
  return 0;
}

/** 一个条目可以有好几个名字（本地化名、英文名、别名），取最高分 */
export function best(q: string, all: Keys[]) {
  let m = 0;
  for (const k of all) m = Math.max(m, score(q, k));
  return m;
}
