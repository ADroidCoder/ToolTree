// 截图标注里不依赖画布的部分。运行：npm test
import { test } from "node:test";
import assert from "node:assert/strict";
import { isLight, isTrivial, normRect } from "../src/lib/shot/draw.ts";

test("从任意方向拖出来的矩形都规整成左上角加宽高", () => {
  assert.deepEqual(normRect(100, 80, 20, 10), { x: 20, y: 10, w: 80, h: 70 });
  assert.deepEqual(normRect(5, 5, 5, 9), { x: 5, y: 5, w: 0, h: 4 });
});

test("手抖点一下产生的形状不保留", () => {
  const base = { color: "#ff3b30", size: 4 };
  assert.ok(isTrivial({ kind: "rect", x1: 10, y1: 10, x2: 11, y2: 11, ...base }));
  assert.ok(!isTrivial({ kind: "arrow", x1: 10, y1: 10, x2: 40, y2: 10, ...base }));
  assert.ok(isTrivial({ kind: "pen", points: [1, 1], ...base }));
  assert.ok(isTrivial({ kind: "text", x: 0, y: 0, text: "  ", ...base }));
  assert.ok(!isTrivial({ kind: "marker", x: 0, y: 0, n: 1, ...base }));
});

test("浅色上用黑字，深色上用白字", () => {
  assert.ok(isLight("#ffffff"));
  assert.ok(isLight("#ffcc00"));
  assert.ok(!isLight("#ff3b30"));
  assert.ok(!isLight("#000000"));
});
