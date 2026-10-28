// 启动器的模糊匹配。运行：npm test
import { test } from "node:test";
import assert from "node:assert/strict";
import { best, keys, score } from "../src/lib/match.ts";

test("英文：前缀、单词开头、首字母缩写", () => {
  const k = keys("Visual Studio Code");
  assert.ok(score("visual", k) > 0);
  assert.ok(score("code", k) > 0);
  assert.ok(score("vsc", k) > 0);
  assert.ok(score("studio co", k) > 0);
  assert.equal(score("xyz", k), 0);
  // 驼峰也能切开
  assert.ok(score("idea", keys("IntelliJ IDEA")) > 0);
  assert.ok(score("tp", keys("TablePlus")) > 0);
});

test("中文：全拼、拼音首字母、混合", () => {
  assert.ok(score("weixin", keys("微信")) > 0);
  assert.ok(score("wx", keys("微信")) > 0);
  assert.ok(score("wei", keys("微信")) > 0);
  assert.ok(score("xtsz", keys("系统设置")) > 0);
  assert.ok(score("qq", keys("QQ音乐")) > 0);
  assert.ok(score("qqyinyue", keys("QQ音乐")) > 0);
  assert.ok(score("wyy", keys("网易云音乐")) > 0);
  assert.ok(score("设置", keys("系统设置")) > 0);
});

test("排序：完全相同 > 前缀 > 单词开头 > 包含", () => {
  const exact = score("notes", keys("Notes"));
  const prefix = score("note", keys("Notes"));
  const word = score("note", keys("Sticky Notes"));
  const sub = score("ote", keys("Notes"));
  assert.ok(exact > prefix && prefix > word && word > sub && sub > 0);
});

test("多个名字取最高分：本地化名和英文名都能搜到", () => {
  const all = [keys("系统设置"), keys("System Settings")];
  assert.ok(best("system", all) > 0);
  assert.ok(best("xtsz", all) > 0);
});
