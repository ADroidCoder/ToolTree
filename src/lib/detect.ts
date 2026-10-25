// 识别一段文本是什么：JSON、时间戳、JWT、Base64、网址、颜色……
// 剪贴板历史和搜索框都用它来推荐“用哪个工具打开”，并直接给出一个预览结果。

import { base64ToBytes, formatDateTime, utf8 } from "./util.ts";

export interface Detection {
  tool: string;
  /** 识别出的类型，比如“JSON”“时间戳（秒）” */
  label: string;
  /** 直接能看的转换结果，比如时间戳对应的日期 */
  preview?: string;
}

function tryJson(s: string) {
  if (!/^[[{]/.test(s)) return false;
  try {
    JSON.parse(s);
    return true;
  } catch {
    return false;
  }
}

function tryBase64(s: string) {
  if (s.length < 8 || s.length > 1_000_000 || !/^[A-Za-z0-9+/_-]+={0,2}$/.test(s)) return null;
  // 纯字母或纯数字多半是普通单词或 ID，不当成 Base64
  if (/^[A-Za-z]+$/.test(s) || /^\d+$/.test(s)) return null;
  try {
    const text = utf8.decode(base64ToBytes(s), true);
    // 解出来要是正常的可读文本
    if (!text || /[\x00-\x08\x0e-\x1f\x7f]/.test(text)) return null;
    return text;
  } catch {
    return null;
  }
}

export function detect(raw: string): Detection[] {
  const s = raw.trim();
  if (!s || s.length > 2_000_000) return [];
  const out: Detection[] = [];
  const oneLine = !s.includes("\n");

  if (tryJson(s)) out.push({ tool: "json", label: "JSON" });

  if (oneLine) {
    if (/^(Bearer\s+)?eyJ[\w-]+\.eyJ[\w-]*\.[\w-]*$/.test(s)) {
      out.push({ tool: "jwt", label: "JWT" });
      return out;
    }

    if (/^\d{10}$|^\d{13}$/.test(s)) {
      const ms = s.length === 10 ? Number(s) * 1000 : Number(s);
      // 只认 2000 年到 2100 年之间的，避免把普通数字当成时间戳
      if (ms > 946_684_800_000 && ms < 4_102_444_800_000) {
        out.push({ tool: "timestamp", label: s.length === 10 ? "时间戳（秒）" : "时间戳（毫秒）", preview: formatDateTime(ms) });
      }
    }

    if (/^https?:\/\/\S+$/i.test(s)) {
      let preview: string | undefined;
      if (/%[0-9a-f]{2}/i.test(s)) {
        try {
          preview = decodeURIComponent(s);
        } catch {
          // 不完整的编码就不预览
        }
      }
      out.push({ tool: "url", label: "网址", preview });
    } else if (/%[0-9a-f]{2}/i.test(s)) {
      try {
        out.push({ tool: "url", label: "URL 编码", preview: decodeURIComponent(s.replace(/\+/g, " ")) });
      } catch {
        // 不是 URL 编码
      }
    }

    if (/^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i.test(s)) {
      out.push({ tool: "uuid", label: `UUID v${s[14]}` });
    } else if (/^#(?:[0-9a-f]{3,4}|[0-9a-f]{6}|[0-9a-f]{8})$/i.test(s) || /^(rgb|hsl)a?\(.+\)$/i.test(s)) {
      out.push({ tool: "color", label: "颜色", preview: s });
    } else if (/^0x[0-9a-f]+$/i.test(s) || /^0b[01]+$/i.test(s)) {
      const v = BigInt(s);
      out.push({ tool: "radix", label: s[1].toLowerCase() === "x" ? "十六进制数" : "二进制数", preview: v.toString() });
    } else if (/^(?:[a-z][a-z\d]*)(?:[A-Z][a-z\d]*)+$|^[a-z\d]+(?:_[a-z\d]+)+$|^[a-z\d]+(?:-[a-z\d]+)+$/.test(s) && s.length < 80) {
      out.push({ tool: "case", label: "变量名" });
    }

    const b64 = !out.length && tryBase64(s.replace(/^data:[^,]*;base64,/, ""));
    if (b64) out.push({ tool: "base64", label: "Base64", preview: b64.length > 200 ? b64.slice(0, 200) + "…" : b64 });
  }
  return out;
}
