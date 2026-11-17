// AI 剧情总结：OpenAI 兼容 /chat/completions（DeepSeek 等）
// 只发送片名/集名/类型/年份/演员/简介等元信息，不发送播放地址，也不后台自动调用
use crate::error::{AppError, AppResult};
use crate::settings::AiSettings;
use serde_json::{json, Value};

const SYSTEM_PROMPT: &str = "你是影视剧情总结助手。根据用户提供的节目信息，用中文输出本集剧情梗概，要求：\
1. 150-250 字；2. 只基于给出的信息，不编造未提及的情节，不给后续集数剧透；\
3. 语气自然，直接输出梗概正文，不要标题、前言和列表符号。";

pub async fn ai_summary(
    client: &reqwest::Client,
    cfg: &AiSettings,
    name: &str,
    episode: &str,
    type_name: &str,
    year: &str,
    actor: &str,
    content: &str,
) -> AppResult<String> {
    let key = cfg.api_key.trim();
    if key.is_empty() {
        return Err(AppError::msg("尚未配置 AI API Key（设置页 → AI 总结）"));
    }
    let base = {
        let b = cfg.base_url.trim().trim_end_matches('/');
        if b.is_empty() {
            "https://api.deepseek.com".to_string()
        } else {
            b.to_string()
        }
    };
    let url = format!("{base}/chat/completions");

    let mut info = format!("片名：{name}");
    if !episode.trim().is_empty() {
        info.push_str(&format!("\n本集：{episode}"));
    }
    if !type_name.trim().is_empty() {
        info.push_str(&format!("\n类型：{type_name}"));
    }
    if !year.trim().is_empty() {
        info.push_str(&format!("\n年份：{year}"));
    }
    if !actor.trim().is_empty() {
        info.push_str(&format!("\n演员：{actor}"));
    }
    if !content.trim().is_empty() {
        let brief: String = content.chars().take(600).collect();
        info.push_str(&format!("\n站点简介：{brief}"));
    }

    let body = json!({
        "model": if cfg.model.trim().is_empty() { "deepseek-flash".to_string() } else { cfg.model.trim().to_string() },
        "messages": [
            { "role": "system", "content": SYSTEM_PROMPT },
            { "role": "user", "content": info }
        ],
        "stream": false,
        "temperature": 0.7,
    });

    let resp = client
        .post(&url)
        .header("Authorization", format!("Bearer {key}"))
        .header("Content-Type", "application/json")
        .timeout(std::time::Duration::from_secs(60))
        .json(&body)
        .send()
        .await
        .map_err(|e| AppError::Http(format!("AI 请求失败: {e}")))?;

    let status = resp.status();
    let text = resp
        .text()
        .await
        .map_err(|e| AppError::Http(format!("AI 响应读取失败: {e}")))?;
    if !status.is_success() {
        // 尽量带出接口的错误信息
        let msg = serde_json::from_str::<Value>(&text)
            .ok()
            .and_then(|v| v["error"]["message"].as_str().map(|s| s.to_string()))
            .unwrap_or_else(|| text.chars().take(200).collect());
        return Err(AppError::Http(format!("AI 接口返回 HTTP {status}: {msg}")));
    }
    let v: Value = serde_json::from_str(&text)
        .map_err(|_| AppError::msg("AI 响应不是合法 JSON"))?;
    let content_out = v["choices"][0]["message"]["content"]
        .as_str()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .ok_or_else(|| AppError::msg("AI 响应中没有内容"))?;
    Ok(content_out)
}
