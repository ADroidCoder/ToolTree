// 启动器里的计算器：输入算式直接出结果。自己解析，不用 eval，输入什么都不会被当成代码执行。
//
// 支持 + - * / % ^（乘方）、括号、隐式乘法（2pi、3(4+5)）、科学计数法、0x / 0b / 0o 进制，
// 函数 sqrt cbrt abs round floor ceil sin cos tan asin acos atan ln log log2 exp，常量 pi e。
// 全角符号和 × ÷ 也认，中文输入法下不用切换。

const FUNCS: Record<string, (x: number) => number> = {
  sqrt: Math.sqrt,
  cbrt: Math.cbrt,
  abs: Math.abs,
  round: Math.round,
  floor: Math.floor,
  ceil: Math.ceil,
  sin: Math.sin,
  cos: Math.cos,
  tan: Math.tan,
  asin: Math.asin,
  acos: Math.acos,
  atan: Math.atan,
  ln: Math.log,
  log: Math.log10,
  log2: Math.log2,
  exp: Math.exp,
};

const CONSTS: Record<string, number> = { pi: Math.PI, π: Math.PI, e: Math.E };

type Token = { t: "num"; v: number } | { t: "op"; v: string } | { t: "id"; v: string };

function normalize(s: string) {
  return s
    .replace(/[（]/g, "(")
    .replace(/[）]/g, ")")
    .replace(/[×＊]/g, "*")
    // 3x4、(2+1) x 5 里的 x 是乘号；0x1f 是十六进制，不动
    .replace(/(?<!(?:^|[^\d.])0)(?<=[\d)]\s*)[xX](?=\s*[\d(])/g, "*")
    .replace(/[÷／]/g, "/")
    .replace(/[＋]/g, "+")
    .replace(/[－−]/g, "-")
    .replace(/[％]/g, "%")
    .replace(/[＾]/g, "^")
    .replace(/[，,_](?=\d{3}(?!\d))/g, "")
    .replace(/[。]/g, ".")
    .replace(/[０-９]/g, (c) => String.fromCharCode(c.charCodeAt(0) - 0xfee0));
}

function tokenize(src: string): Token[] | null {
  const s = normalize(src);
  const out: Token[] = [];
  let i = 0;
  while (i < s.length) {
    const c = s[i];
    if (/\s/.test(c)) {
      i++;
      continue;
    }
    const radix = /^0([xbo])([0-9a-f]+)/i.exec(s.slice(i));
    if (radix) {
      const base = { x: 16, b: 2, o: 8 }[radix[1].toLowerCase() as "x" | "b" | "o"];
      const v = parseInt(radix[2], base);
      if (Number.isNaN(v) || ![...radix[2]].every((d) => parseInt(d, base) < base)) return null;
      out.push({ t: "num", v });
      i += radix[0].length;
      continue;
    }
    const num = /^(\d+\.?\d*|\.\d+)(e[+-]?\d+)?/i.exec(s.slice(i));
    if (num) {
      out.push({ t: "num", v: parseFloat(num[0]) });
      i += num[0].length;
      continue;
    }
    if ("+-*/%^()".includes(c)) {
      // ** 也当乘方
      if (c === "*" && s[i + 1] === "*") {
        out.push({ t: "op", v: "^" });
        i += 2;
        continue;
      }
      out.push({ t: "op", v: c });
      i++;
      continue;
    }
    const id = /^([a-z][a-z0-9]*|π)/i.exec(s.slice(i));
    if (id) {
      out.push({ t: "id", v: id[0].toLowerCase() });
      i += id[0].length;
      continue;
    }
    return null;
  }
  return out;
}

class Parser {
  private i = 0;
  private toks: Token[];
  constructor(toks: Token[]) {
    this.toks = toks;
  }

  parse(): number | null {
    const v = this.expr();
    return v !== null && this.i === this.toks.length ? v : null;
  }

  private peek() {
    return this.toks[this.i];
  }

  private isOp(v: string) {
    const t = this.peek();
    return t?.t === "op" && t.v === v;
  }

  // expr := term (('+' | '-') term)*
  private expr(): number | null {
    let a = this.term();
    while (a !== null && (this.isOp("+") || this.isOp("-"))) {
      const op = (this.toks[this.i++] as { v: string }).v;
      const b = this.term();
      if (b === null) return null;
      a = op === "+" ? a + b : a - b;
    }
    return a;
  }

  // term := unary (('*' | '/' | '%') unary | 隐式乘法 power)*
  private term(): number | null {
    let a = this.unary();
    while (a !== null) {
      if (this.isOp("*") || this.isOp("/") || this.isOp("%")) {
        const op = (this.toks[this.i++] as { v: string }).v;
        // 末尾的 % 是百分号：50% → 0.5，200*15% → 30
        if (op === "%" && this.atTermEnd()) {
          a = a / 100;
          continue;
        }
        const b = this.unary();
        if (b === null) return null;
        a = op === "*" ? a * b : op === "/" ? a / b : a % b;
      } else if (this.peek() && (this.peek().t !== "op" || this.isOp("("))) {
        // 2pi、3(4+5)、2sqrt(2)
        const b = this.power();
        if (b === null) return null;
        a = a * b;
      } else break;
    }
    return a;
  }

  private atTermEnd() {
    const t = this.peek();
    return !t || (t.t === "op" && (t.v === "+" || t.v === "-" || t.v === ")"));
  }

  // unary := ('-' | '+') unary | power
  private unary(): number | null {
    if (this.isOp("-")) {
      this.i++;
      const v = this.unary();
      return v === null ? null : -v;
    }
    if (this.isOp("+")) {
      this.i++;
      return this.unary();
    }
    return this.power();
  }

  // power := atom ('^' unary)?   右结合：2^3^2 = 2^9
  private power(): number | null {
    const a = this.atom();
    if (a === null) return null;
    if (this.isOp("^")) {
      this.i++;
      const b = this.unary();
      return b === null ? null : Math.pow(a, b);
    }
    return a;
  }

  private atom(): number | null {
    const t = this.peek();
    if (!t) return null;
    if (t.t === "num") {
      this.i++;
      return t.v;
    }
    if (t.t === "op" && t.v === "(") {
      this.i++;
      const v = this.expr();
      if (v === null) return null;
      // 末尾少写的右括号自动补上
      if (this.isOp(")")) this.i++;
      else if (this.peek()) return null;
      return v;
    }
    if (t.t === "id") {
      this.i++;
      if (t.v in CONSTS) return CONSTS[t.v];
      const f = FUNCS[t.v];
      if (!f) return null;
      const arg = this.power();
      return arg === null ? null : f(arg);
    }
    return null;
  }
}

export interface CalcResult {
  value: number;
  /** 显示和复制用的结果 */
  text: string;
}

/** 算一个式子。不像算式（只有一个数、普通单词）或者算不出来时返回 null */
export function calc(input: string): CalcResult | null {
  const s = input.trim();
  if (!s || s.length > 200) return null;
  const toks = tokenize(s);
  if (!toks || !toks.length) return null;
  // 至少要有运算符或函数，单独一个数字留给时间戳、进制识别
  const hasOp =
    toks.some((t) => t.t === "op" && t.v !== "(" && t.v !== ")") ||
    toks.some((t) => t.t === "id" && (t.v in FUNCS || (t.v in CONSTS && toks.length > 1)));
  if (!hasOp) return null;
  // 单独的 "-5" 不算
  if (toks.length === 2 && toks[0].t === "op" && toks[0].v === "-") return null;
  const value = new Parser(toks).parse();
  if (value === null || !Number.isFinite(value)) return null;
  return { value, text: formatNumber(value) };
}

/** 去掉浮点误差（0.1+0.2 → 0.3），太大太小的用科学计数法 */
export function formatNumber(v: number) {
  if (Object.is(v, -0)) v = 0;
  const abs = Math.abs(v);
  if (abs !== 0 && (abs >= 1e21 || abs < 1e-9)) return v.toPrecision(12).replace(/\.?0+e/, "e");
  const r = parseFloat(v.toPrecision(14));
  return String(r);
}

/** 加上千分位，只用来显示 */
export function groupDigits(text: string) {
  if (/e/i.test(text)) return text;
  const [int, frac] = text.split(".");
  const g = int.replace(/\B(?=(\d{3})+(?!\d))/g, ",");
  return frac ? `${g}.${frac}` : g;
}
