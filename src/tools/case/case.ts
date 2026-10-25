/** 把任意风格的名字拆成单词：userID、user_id、User-Id、user id 都拆成 user / id */
export function words(s: string): string[] {
  return s
    .replace(/([a-z\d])([A-Z])/g, "$1 $2")
    .replace(/([A-Z]+)([A-Z][a-z])/g, "$1 $2")
    .split(/[^A-Za-z\d一-龥]+/)
    .filter(Boolean)
    .map((w) => w.toLowerCase());
}

const cap = (w: string) => w.charAt(0).toUpperCase() + w.slice(1);

export const CASES: { name: string; fn: (w: string[]) => string }[] = [
  { name: "camelCase", fn: (w) => w.map((x, i) => (i ? cap(x) : x)).join("") },
  { name: "PascalCase", fn: (w) => w.map(cap).join("") },
  { name: "snake_case", fn: (w) => w.join("_") },
  { name: "SCREAMING_SNAKE", fn: (w) => w.join("_").toUpperCase() },
  { name: "kebab-case", fn: (w) => w.join("-") },
  { name: "COBOL-CASE", fn: (w) => w.join("-").toUpperCase() },
  { name: "dot.case", fn: (w) => w.join(".") },
  { name: "path/case", fn: (w) => w.join("/") },
  { name: "Title Case", fn: (w) => w.map(cap).join(" ") },
  { name: "Sentence case", fn: (w) => cap(w.join(" ")) },
  { name: "lower case", fn: (w) => w.join(" ") },
  { name: "UPPER CASE", fn: (w) => w.join(" ").toUpperCase() },
];

/** 字符、字节、行、单词、中文字数 */
export function stats(s: string) {
  return {
    chars: [...s].length,
    bytes: new TextEncoder().encode(s).length,
    lines: s ? s.split("\n").length : 0,
    words: (s.match(/[A-Za-z\d_'-]+/g) ?? []).length,
    cjk: (s.match(/[一-龥]/g) ?? []).length,
  };
}
