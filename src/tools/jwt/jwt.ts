import { base64ToBytes, bytesToBase64, utf8 } from "../../lib/util";

export interface Decoded {
  header: Record<string, unknown>;
  payload: unknown;
  signature: string;
}

export function decodeJwt(token: string): { value?: Decoded; error?: string } {
  const parts = token.trim().replace(/^Bearer\s+/i, "").split(".");
  if (parts.length !== 3) return { error: "JWT 应该由三段组成，用 . 分隔" };
  try {
    const header = JSON.parse(utf8.decode(base64ToBytes(parts[0]), true));
    const raw = utf8.decode(base64ToBytes(parts[1]), true);
    let payload: unknown = raw;
    try {
      payload = JSON.parse(raw);
    } catch {
      // 载荷不一定是 JSON
    }
    return { value: { header, payload, signature: parts[2] } };
  } catch {
    return { error: "头部或载荷不是有效的 Base64URL 编码的 JSON" };
  }
}

const HMAC: Record<string, string> = { HS256: "SHA-256", HS384: "SHA-384", HS512: "SHA-512" };

export function canVerify(alg: unknown) {
  return typeof alg === "string" && alg in HMAC;
}

/** 用密钥校验 HS256/384/512 签名 */
export async function verifyHmac(token: string, alg: string, secret: string) {
  const parts = token.trim().replace(/^Bearer\s+/i, "").split(".");
  const key = await crypto.subtle.importKey("raw", utf8.encode(secret), { name: "HMAC", hash: HMAC[alg] }, false, ["sign"]);
  const sig = new Uint8Array(await crypto.subtle.sign("HMAC", key, utf8.encode(parts[0] + "." + parts[1])));
  const expect = bytesToBase64(sig).replace(/\+/g, "-").replace(/\//g, "_").replace(/=+$/, "");
  return expect === parts[2];
}
