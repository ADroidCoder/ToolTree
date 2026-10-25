// 各种 ID 的生成和解析，随机数都来自 crypto.getRandomValues

const hex = (b: Uint8Array) => [...b].map((x) => x.toString(16).padStart(2, "0")).join("");

function format(b: Uint8Array) {
  const h = hex(b);
  return `${h.slice(0, 8)}-${h.slice(8, 12)}-${h.slice(12, 16)}-${h.slice(16, 20)}-${h.slice(20)}`;
}

export function uuidV4() {
  return crypto.randomUUID();
}

/** v7：前 48 位是毫秒时间戳，按生成时间有序，适合当数据库主键 */
export function uuidV7(ms = Date.now()) {
  const b = crypto.getRandomValues(new Uint8Array(16));
  let t = ms;
  for (let i = 5; i >= 0; i--) {
    b[i] = t % 256;
    t = Math.floor(t / 256);
  }
  b[6] = (b[6] & 0x0f) | 0x70;
  b[8] = (b[8] & 0x3f) | 0x80;
  return format(b);
}

const CROCKFORD = "0123456789ABCDEFGHJKMNPQRSTVWXYZ";

/** ULID：10 位时间 + 16 位随机，Crockford Base32 */
export function ulid(ms = Date.now()) {
  let time = "";
  let t = ms;
  for (let i = 0; i < 10; i++) {
    time = CROCKFORD[t % 32] + time;
    t = Math.floor(t / 32);
  }
  const r = crypto.getRandomValues(new Uint8Array(16));
  return time + [...r].map((x) => CROCKFORD[x % 32]).join("");
}

const NANO = "useandom-26T198340PX75pxJACKVERYMINDBUSHWOLF_GQZbfghjklqvwyzrict";

export function nanoid(size = 21) {
  const r = crypto.getRandomValues(new Uint8Array(size));
  return [...r].map((x) => NANO[x & 63]).join("");
}

export interface IdInfo {
  kind: string;
  version?: number;
  /** 能取出时间的 ID（v1、v6、v7、ULID）里的时间，毫秒 */
  time?: number;
}

export function parseId(s: string): IdInfo | null {
  const v = s.trim();
  const m = /^\{?([0-9a-f]{8})-?([0-9a-f]{4})-?([0-9a-f]{4})-?([0-9a-f]{4})-?([0-9a-f]{12})\}?$/i.exec(v);
  if (m) {
    const h = m.slice(1).join("").toLowerCase();
    if (/^0+$/.test(h)) return { kind: "Nil UUID" };
    if (/^f+$/.test(h)) return { kind: "Max UUID" };
    const version = parseInt(h[12], 16);
    const info: IdInfo = { kind: "UUID", version };
    if (version === 7) info.time = parseInt(h.slice(0, 12), 16);
    if (version === 1) {
      // 100 纳秒为单位，从 1582-10-15 开始
      const ticks = BigInt("0x" + h.slice(13, 16) + h.slice(8, 12) + h.slice(0, 8));
      info.time = Number(ticks / 10000n) - 12219292800000;
    }
    if (version === 6) {
      const ticks = BigInt("0x" + h.slice(0, 12) + h.slice(13, 16));
      info.time = Number(ticks / 10000n) - 12219292800000;
    }
    return info;
  }
  if (/^[0-7][0-9A-HJKMNP-TV-Z]{25}$/i.test(v)) {
    let t = 0;
    for (const c of v.slice(0, 10).toUpperCase()) t = t * 32 + CROCKFORD.indexOf(c);
    return { kind: "ULID", time: t };
  }
  return null;
}
