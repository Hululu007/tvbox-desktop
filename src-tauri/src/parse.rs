// 解析接口（parses）兜底：嗅探拿不到直链时，依次尝试订阅里的解析接口
// 典型 parse 接口形如 https://jx.example.com/?url= ，后面拼网页线路地址
use crate::error::{AppError, AppResult};
use regex::Regex;
use reqwest::Client;

const DIRECT_RE: &str = r#"https?://[^\s"'<>\\]+?\.(?:m3u8|mp4|flv|mkv)(?:\?[^\s"'<>\\]*)?"#;

/// 尝试用单个解析接口解析网页线路 -> 直链
pub async fn resolve_parse(
    client: &Client,
    ua: &str,
    parse_url: &str,
    page_url: &str,
) -> AppResult<String> {
    if parse_url.trim().is_empty() || page_url.trim().is_empty() {
        return Err(AppError::msg("解析接口或页面地址为空"));
    }
    // 接口地址一般以 = 或 ?url= 结尾，直接拼接即可；不带斜杠时补一个分隔
    let sep = if parse_url.ends_with('=') || parse_url.ends_with('?') {
        ""
    } else {
        // https://jx.example.com -> https://jx.example.com/?url=page 形式兜底
        if parse_url.contains("?url=") || parse_url.contains("&url=") {
            ""
        } else if parse_url.contains('?') {
            "&url="
        } else {
            "/?url="
        }
    };
    let full = format!("{}{}{}", parse_url.trim(), sep, crate::cms::url_encode(page_url));

    // 1) 若接口 302 跳转且最终落在媒体直链上，直接采用最终地址
    let resp = client
        .get(&full)
        .header("User-Agent", ua)
        .header("Referer", page_url)
        .timeout(std::time::Duration::from_secs(20))
        .send()
        .await
        .map_err(|e| AppError::Http(format!("解析接口请求失败: {e}")))?;
    let final_url = resp.url().to_string();
    let status = resp.status();
    let body = resp.text().await.unwrap_or_default();
    if !status.is_success() {
        return Err(AppError::Http(format!("解析接口返回 HTTP {status}")));
    }
    let direct = Regex::new(DIRECT_RE).expect("static regex");
    if final_url.contains(".m3u8") || final_url.contains(".mp4") || final_url.contains(".flv") {
        return Ok(final_url);
    }

    // 2) 响应文本里找直链
    if let Some(m) = direct.find(&body) {
        return Ok(m.as_str().to_string());
    }

    // 3) JSON url 字段兜底（接口可能返回 {"url": "https://...m3u8", "code": 200}）
    let re_json = Regex::new(r#""url"\s*:\s*"([^"]+)""#).expect("static regex");
    if let Some(c) = re_json.captures(&body) {
        let u = c.get(1).map(|m| m.as_str()).unwrap_or("");
        if u.starts_with("http") && direct.is_match(u) {
            return Ok(u.to_string());
        }
    }

    Err(AppError::msg("解析接口未返回可用直链"))
}

/// 依次尝试解析接口列表，成功返回 (接口名, 直链)
pub async fn try_parse_list(
    client: &Client,
    ua: &str,
    parses: &[crate::models::ParseConfig],
    page_url: &str,
) -> Option<(String, String)> {
    for p in parses {
        if p.url.trim().is_empty() {
            continue;
        }
        if let Ok(u) = resolve_parse(client, ua, &p.url, page_url).await {
            return Some((p.name.clone(), u));
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_direct_re() {
        let re = Regex::new(DIRECT_RE).unwrap();
        assert!(re.is_match("var u=\"https://cdn.a.com/v/1.m3u8?sign=abc\";"));
        assert!(re.is_match("https://x.com/a.mp4"));
        assert!(!re.is_match("https://x.com/page.html"));
    }
}
