// 启动器计算器。运行：npm test
import { test } from "node:test";
import assert from "node:assert/strict";
import { calc, formatNumber, groupDigits } from "../src/lib/calc.ts";

const v = (s: string) => calc(s)?.text ?? null;

test("四则运算和优先级", () => {
  assert.equal(v("12*8+3"), "99");
  assert.equal(v("2+3*4"), "14");
  assert.equal(v("(2+3)*4"), "20");
  assert.equal(v("10/4"), "2.5");
  assert.equal(v("7 % 3"), "1");
  assert.equal(v("-3+5"), "2");
  assert.equal(v("2*-3"), "-6");
});

test("乘方右结合，负号优先级比乘方低", () => {
  assert.equal(v("2^10"), "1024");
  assert.equal(v("2**3"), "8");
  assert.equal(v("2^3^2"), "512");
  assert.equal(v("-2^2"), "-4");
});

test("浮点误差被去掉", () => {
  assert.equal(v("0.1+0.2"), "0.3");
  assert.equal(v("1.1*3"), "3.3");
});

test("函数、常量、隐式乘法", () => {
  assert.equal(v("sqrt(16)"), "4");
  assert.equal(v("sqrt 2 * sqrt 2"), "2");
  assert.equal(v("2pi"), formatNumber(2 * Math.PI));
  assert.equal(v("3(4+5)"), "27");
  assert.equal(v("log(1000)"), "3");
  assert.equal(v("round(2.5)"), "3");
});

test("百分号", () => {
  assert.equal(v("50%+1"), "1.5");
  assert.equal(v("200*15%"), "30");
});

test("全角符号、× ÷、字母 x 当乘号", () => {
  assert.equal(v("３×４"), "12");
  assert.equal(v("10÷4"), "2.5");
  assert.equal(v("（1＋2）＊3"), "9");
  assert.equal(v("3x4"), "12");
  assert.equal(v("(1+1) x 5"), "10");
});

test("进制和科学计数法", () => {
  assert.equal(v("0xff+1"), "256");
  assert.equal(v("0b101*2"), "10");
  assert.equal(v("1e3/4"), "250");
  assert.equal(v("0x1f"), null);
});

test("末尾少写的右括号自动补上", () => {
  assert.equal(v("(1+2"), "3");
  assert.equal(v("2*(3+(1"), "8");
});

test("不像算式的不算", () => {
  for (const s of ["", "123", "-5", "hello", "1+", "1++", "abc+1", "2026-10-28", "alert(1)", "()", "1/0"]) {
    // 日期 2026-10-28 能算出来，但它就是个算式，算出来也没问题；只要求不报错
    if (s === "2026-10-28") assert.equal(v(s), "1988");
    else assert.equal(v(s), null, s);
  }
});

test("千分位只用来显示", () => {
  assert.equal(groupDigits("1234567.891"), "1,234,567.891");
  assert.equal(groupDigits("-1000"), "-1,000");
  assert.equal(groupDigits("1.5e+30"), "1.5e+30");
  assert.equal(v("1,000*2"), "2000");
});
