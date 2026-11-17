// 直播电视：m3u / txt 解析 + 线路测速
use crate::error::AppResult;
use crate::http::fetch_text;
use crate::models::{LiveChannel, LiveChannelGroup, LineSpeed};
use crate::state::AppState;
use futures::stream::{self, StreamExt};
use tauri::Emitter;

/// txt 格式：
///   央视,#genre#
///   CCTV1,http://...#http://...
/// m3u 格式：#EXTINF:-1 group-title="央视" tvg-name="CCTV1",CCTV1
fn parse_live_text(text: &str) -> Vec<LiveChannelGroup> {
    let mut groups: Vec<LiveChannelGroup> = Vec::new();
    let mut is_m3u = text.trim_start().to_lowercase().starts_with("#extm3u");
    if !is_m3u && text.contains("#EXTINF") {
        is_m3u = true;
    }

    if is_m3u {
        let mut current_group = "未分组".to_string();
        let mut pending_name: Option<(String, String)> = None; // (group, name)
        for raw in text.lines() {
            let line = raw.trim();
            if line.is_empty() {
                continue;
            }
            if line.to_lowercase().starts_with("#extinf") {
                // 提取 group-title
                let group = extract_attr(line, "group-title=")
                    .unwrap_or_else(|| current_group.clone())
                    .trim()
                    .to_string();
                let name = line.rsplit(',').next().unwrap_or("").trim().to_string();
                if !group.is_empty() {
                    current_group = group.clone();
                }
                pending_name = Some((group, name));
            } else if line.starts_with('#') {
                continue;
            } else {
                let url = line.to_string();
                let (group, name) = pending_name.take().unwrap_or_else(|| {
                    (current_group.clone(), String::new())
                });
                if name.is_empty() {
                    continue;
                }
                push_channel(&mut groups, &group, name, url);
            }
        }
    } else {
        for raw in text.lines() {
            let line = raw.trim();
            if line.is_empty() || line.starts_with("//") || line.starts_with('#') && !line.contains(",") {
                continue;
            }
            if line.ends_with("#genre#") {
                let gname = line.trim_end_matches("#genre#").trim_end_matches(',').trim();
                if !gname.is_empty() {
                    groups.push(LiveChannelGroup {
                        group: gname.to_string(),
                        channels: vec![],
                    });
                }
                continue;
            }
            if let Some((name, urls)) = line.split_once(',') {
                let name = name.trim();
                if name.is_empty() || urls.trim().is_empty() {
                    continue;
                }
                for u in urls.split(|c: char| c == '#' || c == '|' || c == '$') {
                    let u = u.trim();
                    if u.starts_with("http://") || u.starts_with("https://") || u.contains("://") {
                        let gname = last_group(&groups);
                        push_channel(&mut groups, &gname, name.to_string(), u.to_string());
                    }
                }
            }
        }
    }
    groups.retain(|g| !g.channels.is_empty());
    groups
}

fn last_group(groups: &[LiveChannelGroup]) -> String {
    groups
        .last()
        .map(|g| g.group.clone())
        .unwrap_or_else(|| "未分组".to_string())
}

fn push_channel(groups: &mut Vec<LiveChannelGroup>, group: &str, name: String, url: String) {
    if group.is_empty() {
        return;
    }
    if let Some(g) = groups.iter_mut().find(|g| g.group == group) {
        if let Some(ch) = g.channels.iter_mut().find(|c| c.name == name) {
            if !ch.lines.contains(&url) {
                ch.lines.push(url);
            }
            return;
        }
        g.channels.push(LiveChannel {
            name,
            lines: vec![url],
        });
    } else {
        groups.push(LiveChannelGroup {
            group: group.to_string(),
            channels: vec![LiveChannel {
                name,
                lines: vec![url],
            }],
        });
    }
}

fn extract_attr(line: &str, key: &str) -> Option<String> {
    let start = line.to_lowercase().find(&key.to_lowercase())? + key.len();
    let rest = &line[start..];
    if rest.starts_with('"') {
        let end = rest[1..].find('"')? + 1;
        Some(rest[1..end].to_string())
    } else {
        Some(rest.split(&[',', ' '][..]).next().unwrap_or("").to_string())
    }
}

/// 从当前激活订阅载入全部直播频道（B 格式远程 url + A 格式内嵌 groups）
pub async fn load_live_groups(state: &AppState) -> AppResult<Vec<LiveChannelGroup>> {
    let cfg = {
        let rt = state.runtime.lock().await;
        state.active_config(&rt)
    };
    let Some(cfg) = cfg else {
        return Ok(vec![]);
    };
    let (ua, timeout) = {
        let s = state
            .settings
            .read()
            .map(|s| (s.effective_ua(), s.request_timeout))
            .unwrap_or((crate::state::USER_AGENT.to_string(), 15));
        s
    };
    let mut out: Vec<LiveChannelGroup> = Vec::new();
    for live in &cfg.lives {
        // A 格式内嵌
        if !live.groups.is_empty() {
            for g in &live.groups {
                let group_name = if g.name.is_empty() { &live.name } else { &g.name };
                for ch in &g.channels {
                    for u in &ch.urls {
                        push_channel(&mut out, group_name, ch.name.clone(), u.clone());
                    }
                }
            }
            continue;
        }
        // B 格式远程源
        let url = live.url.trim();
        if url.is_empty() {
            continue;
        }
        match fetch_text(&state.client, url, &ua, &Default::default(), timeout.min(30)).await {
            Ok(text) => {
                for g in parse_live_text(&text) {
                    for ch in g.channels {
                        for u in ch.lines {
                            push_channel(&mut out, &g.group, ch.name.clone(), u);
                        }
                    }
                }
            }
            Err(_) => continue, // 单个直播源失败不阻塞其它
        }
    }
    Ok(out)
}

/// 并发测速：每条线路 HEAD/GET Range 0-2048，6 秒超时
pub async fn speed_test_lines(
    app: &tauri::AppHandle,
    state: &AppState,
    urls: Vec<String>,
) -> Vec<LineSpeed> {
    let total = urls.len();
    let app = app.clone();
    let ua = state.effective_ua();
    let results: Vec<LineSpeed> = stream::iter(urls.into_iter().enumerate())
        .map(|(idx, url)| {
            let client = state.client.clone();
            let app = app.clone();
            let ua = ua.clone();
            async move {
                let ok_delay = test_line(&client, &ua, &url).await;
                let ls = match ok_delay {
                    Ok(ms) => LineSpeed {
                        url,
                        delay_ms: ms,
                        ok: true,
                    },
                    Err(_) => LineSpeed {
                        url,
                        delay_ms: -1,
                        ok: false,
                    },
                };
                let _ = app.emit(
                    "live:speed",
                    serde_json::json!({ "done": idx + 1, "total": total }),
                );
                ls
            }
        })
        .buffer_unordered(6)
        .collect::<Vec<_>>()
        .await;
    results
}

async fn test_line(client: &reqwest::Client, ua: &str, url: &str) -> AppResult<i64> {
    let start = std::time::Instant::now();
    let resp = client
        .get(url)
        .header("User-Agent", ua)
        .header("Range", "bytes=0-2048")
        .timeout(std::time::Duration::from_secs(6))
        .send()
        .await
        .map_err(|e| crate::error::AppError::Http(e.to_string()))?;
    let status_ok = resp.status().is_success();
    // 消费少量字节确保真实可用
    let _ = resp.bytes().await;
    if !status_ok {
        return Err(crate::error::AppError::Http("非 2xx 响应".into()));
    }
    Ok(start.elapsed().as_millis() as i64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_txt() {
        let text = "央视,#genre#\nCCTV1,http://a/1.m3u8#http://b/2.m3u8\nCCTV2,http://a/2.m3u8\n\n卫视,#genre#\n湖南卫视,http://c/1.m3u8";
        let groups = parse_live_text(text);
        assert_eq!(groups.len(), 2);
        assert_eq!(groups[0].group, "央视");
        assert_eq!(groups[0].channels[0].name, "CCTV1");
        assert_eq!(groups[0].channels[0].lines.len(), 2);
        assert_eq!(groups[0].channels[1].name, "CCTV2");
        assert_eq!(groups[1].group, "卫视");
    }

    #[test]
    fn test_parse_m3u() {
        let text = "#EXTM3U\n#EXTINF:-1 group-title=\"央视\",CCTV1\nhttp://a/1.m3u8\n#EXTINF:-1 group-title=\"央视\",CCTV2\nhttp://a/2.m3u8";
        let groups = parse_live_text(text);
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].group, "央视");
        assert_eq!(groups[0].channels.len(), 2);
    }
}
