// 翻译服务和语言列表，工具页和设置页共用。真正的请求在后端 src-tauri/src/translate.rs

/** 空字符串和 free 一样，是默认的免费服务 */
export type Provider = "" | "free" | "tencent" | "aliyun" | "deepl";

export const PROVIDERS: { value: Provider; label: string }[] = [
  { value: "free", label: "免费（Google / MyMemory）" },
  { value: "tencent", label: "腾讯云" },
  { value: "aliyun", label: "阿里云" },
  { value: "deepl", label: "DeepL" },
];

export const LANGS: { value: string; label: string }[] = [
  { value: "zh", label: "中文" },
  { value: "en", label: "英语" },
  { value: "ja", label: "日语" },
  { value: "ko", label: "韩语" },
  { value: "fr", label: "法语" },
  { value: "de", label: "德语" },
  { value: "es", label: "西班牙语" },
  { value: "ru", label: "俄语" },
];

/** 目标语言选“自动”时：原文是中文就翻成英文，否则翻成中文 */
export function autoTarget(text: string, from: string): string {
  if (from !== "auto") return from === "zh" ? "en" : "zh";
  // 有假名的是日文，不算中文
  if (/[぀-ヿ]/.test(text)) return "zh";
  const han = text.match(/\p{Script=Han}/gu)?.length ?? 0;
  const latin = text.match(/[A-Za-z]/g)?.length ?? 0;
  return han > 0 && han * 3 >= latin ? "en" : "zh";
}
