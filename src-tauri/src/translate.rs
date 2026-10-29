//! 翻译。免费模式先用 Google 的网页接口，不通（国内、被限流）再用 MyMemory，都不需要密钥；
//! 填了自己的密钥后可以用腾讯云、阿里云、DeepL。
//! 语言代码统一用 zh、en、ja 这种两个字母的，auto 表示自动识别，各家的写法在这里转换。

use crate::config::TranslateConfig;
use crate::{err, tr};
use base64::Engine;
use hmac::{Hmac, Mac};
use serde::Serialize;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[derive(Serialize)]
pub struct Translated {
    pub text: String,
    /// 识别出来的原文语言，服务没返回时为空
    pub from: String,
}

pub fn translate(c: &TranslateConfig, text: &str, from: &str, to: &str) -> Result<Translated, String> {
    match c.provider.as_str() {
        "tencent" => tencent(c, text, from, to),
        "aliyun" => aliyun(c, text, from, to),
        "deepl" => deepl(c, text, from, to),
        _ => google(text, from, to).or_else(|_| mymemory(text, from, to)),
    }
}

fn agent() -> ureq::Agent {
    ureq::AgentBuilder::new().timeout(Duration::from_secs(15)).build()
}

/// 读出响应的 JSON。出错的状态码也读，各家的错误信息在 JSON 里
fn read(r: Result<ureq::Response, ureq::Error>) -> Result<Value, String> {
    let resp = match r {
        Ok(r) => r,
        Err(ureq::Error::Status(_, r)) => r,
        Err(e) => return Err(format!("{}：{e}", tr("网络请求失败", "Request failed"))),
    };
    let status = resp.status();
    let body = resp.into_string().map_err(err)?;
    serde_json::from_str(&body).map_err(|_| format!("HTTP {status}: {}", body.chars().take(200).collect::<String>()))
}

/// 按行把长文本切成不超过 max 个字符的几段，分别翻译再拼回去。
/// 返回每段和它后面要不要接换行；单独一行太长时硬切，切开的地方不加换行
fn chunks(text: &str, max: usize) -> Vec<(String, bool)> {
    let mut out: Vec<(String, bool)> = vec![];
    // cur 里已经放了几行（空行也算）、多少个字符
    let mut cur = String::new();
    let mut has = false;
    let mut n = 0;
    for line in text.split('\n') {
        let len = line.chars().count();
        if has && n + 1 + len > max {
            out.push((std::mem::take(&mut cur), true));
            has = false;
            n = 0;
        }
        if len > max {
            let chars: Vec<char> = line.chars().collect();
            for piece in chars.chunks(max) {
                out.push((piece.iter().collect(), false));
            }
            out.last_mut().unwrap().1 = true;
            continue;
        }
        if has {
            cur.push('\n');
            n += 1;
        }
        cur.push_str(line);
        n += len;
        has = true;
    }
    if has || out.is_empty() {
        out.push((cur, true));
    }
    out
}

/// 分段翻译，空白的段原样保留
fn each(text: &str, max: usize, mut f: impl FnMut(&str) -> Result<Translated, String>) -> Result<Translated, String> {
    let mut res = Translated { text: String::new(), from: String::new() };
    for (piece, newline) in chunks(text, max) {
        if piece.trim().is_empty() {
            res.text.push_str(&piece);
        } else {
            let t = f(&piece)?;
            res.text.push_str(&t.text);
            if res.from.is_empty() {
                res.from = t.from;
            }
        }
        if newline {
            res.text.push('\n');
        }
    }
    res.text.pop();
    Ok(res)
}

/// zh-CN、EN 这种转回 zh、en
fn short(lang: &str) -> String {
    lang.split(['-', '_']).next().unwrap_or("").to_lowercase()
}

// ---- 免费 ----

fn google(text: &str, from: &str, to: &str) -> Result<Translated, String> {
    let code = |l: &str| if l == "zh" { "zh-CN".to_string() } else { l.to_string() };
    let v = read(
        agent()
            .post("https://translate.googleapis.com/translate_a/single")
            .query("client", "gtx")
            .query("sl", &code(from))
            .query("tl", &code(to))
            .query("dt", "t")
            .send_form(&[("q", text)]),
    )?;
    // [[["译文", "原文", ...], ...], null, "en", ...]
    let parts = v[0].as_array().ok_or_else(|| tr("Google 翻译返回了看不懂的结果", "Unexpected response from Google").to_string())?;
    let text = parts.iter().filter_map(|p| p[0].as_str()).collect();
    Ok(Translated { text, from: v[2].as_str().map(short).unwrap_or_default() })
}

fn mymemory(text: &str, from: &str, to: &str) -> Result<Translated, String> {
    let code = |l: &str| match l {
        "auto" => "autodetect".to_string(),
        "zh" => "zh-CN".to_string(),
        l => l.to_string(),
    };
    let pair = format!("{}|{}", code(from), code(to));
    // 每次最多 500 个字符
    each(text, 450, |q| {
        let v = read(agent().get("https://api.mymemory.translated.net/get").query("q", q).query("langpair", &pair).call())?;
        if v["quotaFinished"].as_bool() == Some(true) {
            return Err(tr("MyMemory 今天的免费额度用完了，可以在设置里填自己的翻译密钥", "MyMemory's free quota for today is used up. You can add your own API key in Settings").into());
        }
        // 状态有时是数字，有时是字符串
        let status = v["responseStatus"].as_i64().or_else(|| v["responseStatus"].as_str().and_then(|s| s.parse().ok()));
        if status != Some(200) {
            return Err(format!("MyMemory: {}", v["responseDetails"].as_str().unwrap_or("")));
        }
        Ok(Translated {
            text: v["responseData"]["translatedText"].as_str().unwrap_or("").into(),
            from: v["responseData"]["detectedLanguage"].as_str().map(short).unwrap_or_default(),
        })
    })
}

// ---- 腾讯云：TC3-HMAC-SHA256 签名 ----

fn hmac256(key: &[u8], msg: &str) -> Vec<u8> {
    let mut m = Hmac::<Sha256>::new_from_slice(key).unwrap();
    m.update(msg.as_bytes());
    m.finalize().into_bytes().to_vec()
}

fn sha256_hex(s: &str) -> String {
    hex::encode(Sha256::digest(s.as_bytes()))
}

fn tencent(c: &TranslateConfig, text: &str, from: &str, to: &str) -> Result<Translated, String> {
    if c.tencent_id.is_empty() || c.tencent_key.is_empty() {
        return Err(tr("还没有填腾讯云的 SecretId 和 SecretKey，在设置里填", "Tencent Cloud SecretId and SecretKey are not set. Add them in Settings").into());
    }
    const HOST: &str = "tmt.tencentcloudapi.com";
    // 每次最多 6000 个字符
    each(text, 5000, |q| {
        let payload = json!({ "SourceText": q, "Source": from, "Target": to, "ProjectId": 0 }).to_string();
        let now = chrono::Utc::now();
        let ts = now.timestamp().to_string();
        let date = now.format("%Y-%m-%d").to_string();
        let canonical = format!(
            "POST\n/\n\ncontent-type:application/json; charset=utf-8\nhost:{HOST}\nx-tc-action:texttranslate\n\ncontent-type;host;x-tc-action\n{}",
            sha256_hex(&payload)
        );
        let scope = format!("{date}/tmt/tc3_request");
        let to_sign = format!("TC3-HMAC-SHA256\n{ts}\n{scope}\n{}", sha256_hex(&canonical));
        let k = hmac256(format!("TC3{}", c.tencent_key).as_bytes(), &date);
        let k = hmac256(&k, "tmt");
        let k = hmac256(&k, "tc3_request");
        let sig = hex::encode(hmac256(&k, &to_sign));
        let auth = format!("TC3-HMAC-SHA256 Credential={}/{scope}, SignedHeaders=content-type;host;x-tc-action, Signature={sig}", c.tencent_id);
        let v = read(
            agent()
                .post(&format!("https://{HOST}"))
                .set("Authorization", &auth)
                .set("Content-Type", "application/json; charset=utf-8")
                .set("X-TC-Action", "TextTranslate")
                .set("X-TC-Timestamp", &ts)
                .set("X-TC-Version", "2018-03-21")
                .set("X-TC-Region", "ap-guangzhou")
                .send_string(&payload),
        )?;
        let r = &v["Response"];
        if let Some(msg) = r["Error"]["Message"].as_str() {
            return Err(format!("{}: {msg}", tr("腾讯云", "Tencent Cloud")));
        }
        Ok(Translated { text: r["TargetText"].as_str().unwrap_or("").into(), from: r["Source"].as_str().map(short).unwrap_or_default() })
    })
}

// ---- 阿里云：RPC 风格的 HMAC-SHA1 签名 ----

/// RFC 3986 的百分号编码，阿里云签名要求的格式
fn pct(s: &str) -> String {
    let mut out = String::new();
    for b in s.bytes() {
        if b.is_ascii_alphanumeric() || b"-_.~".contains(&b) {
            out.push(b as char);
        } else {
            out.push_str(&format!("%{b:02X}"));
        }
    }
    out
}

fn nonce() -> String {
    static N: AtomicU64 = AtomicU64::new(0);
    let t = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_nanos();
    format!("{t}{}", N.fetch_add(1, Ordering::Relaxed))
}

fn aliyun(c: &TranslateConfig, text: &str, from: &str, to: &str) -> Result<Translated, String> {
    if c.aliyun_id.is_empty() || c.aliyun_secret.is_empty() {
        return Err(tr("还没有填阿里云的 AccessKey ID 和 Secret，在设置里填", "Alibaba Cloud AccessKey ID and Secret are not set. Add them in Settings").into());
    }
    // 每次最多 5000 个字符
    each(text, 4500, |q| {
        let ts = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
        let nonce = nonce();
        let mut params = vec![
            ("AccessKeyId", c.aliyun_id.as_str()),
            ("Action", "TranslateGeneral"),
            ("Format", "JSON"),
            ("FormatType", "text"),
            ("Scene", "general"),
            ("SignatureMethod", "HMAC-SHA1"),
            ("SignatureNonce", &nonce),
            ("SignatureVersion", "1.0"),
            ("SourceLanguage", from),
            ("SourceText", q),
            ("TargetLanguage", to),
            ("Timestamp", &ts),
            ("Version", "2018-10-12"),
        ];
        params.sort_by_key(|p| p.0);
        let query = params.iter().map(|(k, v)| format!("{}={}", pct(k), pct(v))).collect::<Vec<_>>().join("&");
        let to_sign = format!("POST&%2F&{}", pct(&query));
        let mut m = Hmac::<sha1::Sha1>::new_from_slice(format!("{}&", c.aliyun_secret).as_bytes()).unwrap();
        m.update(to_sign.as_bytes());
        let sig = base64::engine::general_purpose::STANDARD.encode(m.finalize().into_bytes());
        params.push(("Signature", &sig));
        let v = read(agent().post("https://mt.cn-hangzhou.aliyuncs.com/").send_form(&params))?;
        // 成功时 Code 是 "200"（有时是数字 200）
        let code = v["Code"].as_str().map(String::from).or_else(|| v["Code"].as_i64().map(|n| n.to_string()));
        if code.as_deref() != Some("200") {
            return Err(format!("{}: {}", tr("阿里云", "Alibaba Cloud"), v["Message"].as_str().unwrap_or(&v.to_string())));
        }
        let d = &v["Data"];
        Ok(Translated { text: d["Translated"].as_str().unwrap_or("").into(), from: d["DetectedLanguage"].as_str().map(short).unwrap_or_default() })
    })
}

// ---- DeepL ----

fn deepl(c: &TranslateConfig, text: &str, from: &str, to: &str) -> Result<Translated, String> {
    let key = c.deepl_key.trim();
    if key.is_empty() {
        return Err(tr("还没有填 DeepL 的密钥，在设置里填", "DeepL API key is not set. Add it in Settings").into());
    }
    // 免费版的密钥以 :fx 结尾，用另一个地址
    let host = if key.ends_with(":fx") { "api-free.deepl.com" } else { "api.deepl.com" };
    let target = match to {
        "zh" => "ZH-HANS".to_string(),
        "en" => "EN-US".to_string(),
        l => l.to_uppercase(),
    };
    let mut body = json!({ "text": [text], "target_lang": target });
    if from != "auto" {
        body["source_lang"] = json!(from.to_uppercase());
    }
    let v = read(
        agent()
            .post(&format!("https://{host}/v2/translate"))
            .set("Authorization", &format!("DeepL-Auth-Key {key}"))
            .send_json(body),
    )?;
    let t = &v["translations"][0];
    if t.is_null() {
        return Err(format!("DeepL: {}", v["message"].as_str().unwrap_or(&v.to_string())));
    }
    Ok(Translated { text: t["text"].as_str().unwrap_or("").into(), from: t["detected_source_language"].as_str().map(short).unwrap_or_default() })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn join(c: Vec<(String, bool)>) -> String {
        let mut s: String = c.into_iter().map(|(p, nl)| if nl { p + "\n" } else { p }).collect();
        s.pop();
        s
    }

    #[test]
    fn chunks_keep_text() {
        for text in ["", "a", "a\nb\n\nc", "aaaaaaaaaa\nbb\ncccccccccccccccccccccc\n", "aaaaaaaaaa\n\nbb", "\n\n", "中文中文中文中文\n英文"] {
            let c = chunks(text, 5);
            assert!(c.iter().all(|(p, _)| p.chars().count() <= 5), "{c:?}");
            assert_eq!(join(c), text);
        }
    }

    #[test]
    fn aliyun_pct() {
        assert_eq!(pct("a b*~/中"), "a%20b%2A~%2F%E4%B8%AD");
    }
}
