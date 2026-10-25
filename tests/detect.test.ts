// 剪贴板内容识别。运行：npm test
import { test } from "node:test";
import assert from "node:assert/strict";
import { detect } from "../src/lib/detect.ts";

const tools = (s: string) => detect(s).map((d) => d.tool);

test("JSON", () => {
  assert.deepEqual(tools('{"a": 1}'), ["json"]);
  assert.deepEqual(tools("[1, 2]"), ["json"]);
  assert.deepEqual(tools("{not json}"), []);
});

test("时间戳只认合理范围内的 10 位和 13 位数字", () => {
  const d = detect("1761400000");
  assert.equal(d[0].tool, "timestamp");
  assert.match(d[0].preview!, /^2025-10-2/);
  assert.equal(detect("1761400000123")[0].tool, "timestamp");
  assert.deepEqual(tools("1234567890123456"), []);
  assert.deepEqual(tools("0000000001"), []);
});

test("JWT 优先于 Base64", () => {
  const jwt = "eyJhbGciOiJIUzI1NiJ9.eyJzdWIiOiIxMjM0In0.abc_def-123";
  assert.deepEqual(tools(jwt), ["jwt"]);
  assert.deepEqual(tools("Bearer " + jwt), ["jwt"]);
});

test("网址和 URL 编码", () => {
  assert.deepEqual(tools("https://example.com/a?b=1"), ["url"]);
  const d = detect("a%20b%E4%B8%AD");
  assert.equal(d[0].tool, "url");
  assert.equal(d[0].preview, "a b中");
});

test("Base64 要解得出可读文本，普通单词不算", () => {
  const d = detect(btoa("hello world"));
  assert.equal(d[0].tool, "base64");
  assert.equal(d[0].preview, "hello world");
  assert.deepEqual(tools("password"), []);
  assert.deepEqual(tools("12345678"), []);
});

test("颜色、UUID、进制、变量名", () => {
  assert.deepEqual(tools("#ff6600"), ["color"]);
  assert.deepEqual(tools("rgb(1, 2, 3)"), ["color"]);
  assert.equal(detect("0190a6e4-5a1b-7c3d-8e9f-0123456789ab")[0].label, "UUID v7");
  assert.equal(detect("0xff")[0].preview, "255");
  assert.deepEqual(tools("userProfileId"), ["case"]);
  assert.deepEqual(tools("user_profile_id"), ["case"]);
});

test("多行文本只认 JSON", () => {
  assert.deepEqual(tools("hello\nworld"), []);
  assert.deepEqual(tools('{\n  "a": 1\n}'), ["json"]);
});
