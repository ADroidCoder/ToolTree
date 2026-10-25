// 各工具里的纯函数。运行：npm test
import { test } from "node:test";
import assert from "node:assert/strict";
import { parseJson, sortKeys, toTypeScript } from "../src/tools/json/json.ts";
import { formatInZone, guessUnit, parseInput } from "../src/tools/timestamp/time.ts";
import { CASES, stats, words } from "../src/tools/case/case.ts";
import { parseId, ulid, uuidV7 } from "../src/tools/uuid/ids.ts";

test("JSON 出错时给出行列", () => {
  const r = parseJson('{\n  "a": 1,\n}');
  assert.ok(r.error);
  assert.match(r.error!, /3:1/);
});

test("按键名排序是递归的", () => {
  assert.equal(JSON.stringify(sortKeys({ b: 1, a: { d: 1, c: [{ z: 1, y: 2 }] } })), '{"a":{"c":[{"y":2,"z":1}],"d":1},"b":1}');
});

test("JSON 生成 TypeScript 类型：数组里的对象合并，缺的字段可选", () => {
  const ts = toTypeScript({ name: "a", tags: ["x"], users: [{ id: 1, nick: "n" }, { id: 2 }], extra: null });
  assert.match(ts, /export interface Root \{/);
  assert.match(ts, /tags: string\[\];/);
  assert.match(ts, /users: User\[\];/);
  assert.match(ts, /export interface User \{\n  id: number;\n  nick\?: string;\n\}/);
  assert.match(ts, /extra: null;/);
});

test("时间戳按位数判断单位", () => {
  assert.equal(guessUnit("1761400000"), "s");
  assert.equal(guessUnit("1761400000000"), "ms");
  assert.equal(guessUnit("1761400000000000"), "us");
  assert.equal(guessUnit("1761400000000000000"), "ns");
  assert.equal(parseInput("1761400000")!.ms, 1761400000000);
  assert.equal(parseInput("1761400000123456")!.ms, 1761400000123);
});

test("日期的几种写法都能解析", () => {
  const ms = Date.UTC(2026, 0, 5, 4, 0, 0);
  assert.equal(parseInput("2026-01-05T04:00:00Z")!.ms, ms);
  // 不带时区的按本地时间
  const local = new Date(2026, 0, 5, 12, 0, 0).getTime();
  assert.equal(parseInput("2026-01-05 12:00:00")!.ms, local);
  assert.equal(parseInput("2026/1/5 12:00")!.ms, local);
  assert.equal(parseInput("not a date"), null);
});

test("按时区格式化", () => {
  assert.equal(formatInZone(Date.UTC(2026, 9, 25, 16, 0, 0, 5), "Asia/Shanghai"), "2026-10-26 00:00:00.005");
  assert.equal(formatInZone(0, "UTC", false), "1970-01-01 00:00:00");
});

test("命名拆词", () => {
  assert.deepEqual(words("userIDValue"), ["user", "id", "value"]);
  assert.deepEqual(words("HTTPServer_port-number"), ["http", "server", "port", "number"]);
  const camel = CASES.find((c) => c.name === "camelCase")!;
  const snake = CASES.find((c) => c.name === "SCREAMING_SNAKE")!;
  assert.equal(camel.fn(words("user profile id")), "userProfileId");
  assert.equal(snake.fn(words("userProfileId")), "USER_PROFILE_ID");
});

test("字数统计", () => {
  assert.deepEqual(stats("hello 世界\nok"), { chars: 11, bytes: 15, lines: 2, words: 2, cjk: 2 });
});

test("UUID v7 和 ULID 能取回生成时间", () => {
  const ms = Date.UTC(2026, 9, 25);
  const v7 = uuidV7(ms);
  assert.match(v7, /^[0-9a-f]{8}-[0-9a-f]{4}-7[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/);
  assert.deepEqual(parseId(v7), { kind: "UUID", version: 7, time: ms });
  const u = ulid(ms);
  assert.equal(u.length, 26);
  assert.deepEqual(parseId(u), { kind: "ULID", time: ms });
  assert.deepEqual(parseId("6ba7b810-9dad-11d1-80b4-00c04fd430c8")?.version, 1);
  assert.equal(parseId("nope"), null);
});
