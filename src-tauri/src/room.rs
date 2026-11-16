// 房间号频道解析：huya:// roomId、douyin:// roomId -> 真实直播直链
// 说明：这类接口依赖页面结构，随时可能失效；失败时返回明确错误。
use crate::error::{AppError, AppResult};
use crate::state::USER_AGENT;
use serde_json::Value;

/// huya://10188 -> 虎牙 10188 房间 HLS 直链
async fn resolve_huya(room: &str) -> AppResult<String> {
    let url = format!("https://www.huya.com/{room}");
    let resp = reqwest::Client::new()
        .get(&url)
        .header("User-Agent", USER_AGENT)
        .timeout(std::time::Duration::from_secs(12))
        .send()
        .await
        .map_err(|e| AppError::Http(format!("虎牙房间请求失败: {e}")))?;
    if !resp.status().is_success() {
        return Err(AppError::msg(format!("虎牙房间不存在或已关闭 (HTTP {})", resp.status())));
    }
    let html = resp.text().await.map_err(|e| AppError::Http(e.to_string()))?;

    // 页面内 stream: "<json-encoded-json>"，包含 gameStreamInfoList
    let re_stream = regex::Regex::new(r#""stream"\s*:\s*"(.+?)""#).unwrap();
    let caps = re_stream
        .captures(&html)
        .ok_or_else(|| AppError::msg("虎牙页面未找到直播流数据（可能未开播）"))?;
    let encoded = caps.get(1).map(|m| m.as_str()).unwrap_or("");
    // JSON 字符串字面量 -> 内层 JSON 文本
    let inner: String = serde_json::from_str(&format!("\"{encoded}\""))
        .map_err(|_| AppError::msg("虎牙直播流数据解析失败"))?;
    let v: Value = serde_json::from_str(&inner).map_err(|_| AppError::msg("虎牙直播流 JSON 无效"))?;

    let list = v
        .get("gameStreamInfoList")
        .and_then(|x| x.as_array())
        .ok_or_else(|| AppError::msg("虎牙未返回流列表（可能未开播）"))?;
    let first = list
        .first()
        .ok_or_else(|| AppError::msg("虎牙未返回可用线路"))?;
    let hls_url = first.get("sHlsUrl").and_then(|x| x.as_str()).unwrap_or("");
    let stream_name = first.get("sStreamName").and_then(|x| x.as_str()).unwrap_or("");
    let anti_code = first.get("sHlsAntiCode").and_then(|x| x.as_str()).unwrap_or("");
    if hls_url.is_empty() || stream_name.is_empty() {
        return Err(AppError::msg("虎牙流地址字段缺失"));
    }
    Ok(format!("https://{hls_url}/{stream_name}.m3u8?{anti_code}"))
}

/// douyin:// roomId -> 抖音直播间 HLS 直链（实验性，依赖页面结构）
async fn resolve_douyin(room: &str) -> AppResult<String> {
    let url = format!("https://live.douyin.com/{room}");
    let resp = reqwest::Client::new()
        .get(&url)
        .header("User-Agent", USER_AGENT)
        .header("Cookie", "__ac_nonce=0603d0b2c0b26f5f4b8f4d6b4f4d6b4f")
        .timeout(std::time::Duration::from_secs(12))
        .send()
        .await
        .map_err(|e| AppError::Http(format!("抖音房间请求失败: {e}")))?;
    let html = resp.text().await.map_err(|e| AppError::Http(e.to_string()))?;
    // 尽力提取 stream_url 里的 hls
    let re = regex::Regex::new(r#""stream_url"\s*:\s*"(.+?)""#).unwrap();
    let caps = re
        .captures(&html)
        .ok_or_else(|| AppError::msg("抖音页面未找到直播流数据（平台风控或未开播）"))?;
    let encoded = caps.get(1).map(|m| m.as_str()).unwrap_or("");
    let inner: String = serde_json::from_str(&format!("\"{encoded}\""))
        .map_err(|_| AppError::msg("抖音直播流数据解析失败"))?;
    let v: Value = serde_json::from_str(&inner).map_err(|_| AppError::msg("抖音直播流 JSON 无效"))?;
    let common = v.get("common").cloned().unwrap_or(Value::Null);
    let _ = common;
    if let Some(main) = v.get("main") {
        let hls = main
            .get("hls")
            .and_then(|h| h.as_str())
            .or_else(|| {
                main.get("flv")
                    .and_then(|f| f.as_str())
            });
        if let Some(u) = hls {
            return Ok(u.to_string());
        }
    }
    Err(AppError::msg("抖音未返回可用播放地址"))
}

/// 入口：scheme://room 形式解析
pub async fn resolve_room_url(raw: &str) -> AppResult<String> {
    let (scheme, room) = raw
        .split_once("://")
        .ok_or_else(|| AppError::msg("不是合法的房间号地址"))?;
    let room = room.trim().trim_end_matches('/').to_string();
    if room.is_empty() {
        return Err(AppError::msg("房间号为空"));
    }
    match scheme.to_lowercase().as_str() {
        "huya" => resolve_huya(&room).await,
        "douyin" => resolve_douyin(&room).await,
        other => Err(AppError::msg(format!(
            "暂不支持房间号协议: {other}://（支持 huya:// 与 douyin://）"
        ))),
    }
}

/// 判断是否房间号地址
pub fn is_room_url(u: &str) -> bool {
    let lower = u.to_lowercase();
    (lower.starts_with("huya://") || lower.starts_with("douyin://"))
        && !lower.contains(".m3u8")
}
