// 聚合搜索：并发调度全部可搜索站点，事件上报进度
use crate::error::AppResult;
use crate::models::{SearchResultGroup, SiteConfig};
use crate::state::AppState;
use futures::stream::{self, StreamExt};
use tauri::Emitter;

const PER_SITE_TIMEOUT_SECS: u64 = 12;
const MAX_CONCURRENCY: usize = 6;

async fn search_site(state: &AppState, site: SiteConfig, wd: String) -> SearchResultGroup {
    let site_key = site.key.clone();
    let site_name = site.name.clone();
    let client = state.client.clone();
    let task = async {
        if site.is_cms() {
            crate::cms::search_cms(&client, &site, &wd).await
        } else if site.kind == "html" {
            crate::htmlsite::search_html_site(&client, &site, &wd).await
        } else {
            Err(crate::error::AppError::msg("站点类型不支持搜索"))
        }
    };
    match tokio::time::timeout(
        std::time::Duration::from_secs(PER_SITE_TIMEOUT_SECS),
        task,
    )
    .await
    {
        Ok(Ok(items)) => SearchResultGroup {
            site_key,
            site_name,
            ok: true,
            message: String::new(),
            items,
        },
        Ok(Err(e)) => SearchResultGroup {
            site_key,
            site_name,
            ok: false,
            message: e.to_string(),
            items: vec![],
        },
        Err(_) => SearchResultGroup {
            site_key,
            site_name,
            ok: false,
            message: "搜索超时".into(),
            items: vec![],
        },
    }
}

/// 聚合搜索：active 订阅下所有 searchable 站点并发搜索。
/// 每完成一个站点 emit "search:progress" { done, total, site_key, ok, count }。
pub async fn aggregate_search(
    app: &tauri::AppHandle,
    state: &AppState,
    wd: &str,
) -> AppResult<Vec<SearchResultGroup>> {
    let wd = wd.trim().to_string();
    if wd.is_empty() {
        return Ok(vec![]);
    }
    let cfg = {
        let rt = state.runtime.lock().await;
        state.active_config(&rt)
    };
    let Some(cfg) = cfg else {
        return Ok(vec![]);
    };
    let sites: Vec<SiteConfig> = cfg
        .sites
        .into_iter()
        .filter(|s| s.is_searchable())
        .collect();
    let total = sites.len();
    let app_handle = app.clone();

    let results: Vec<SearchResultGroup> = stream::iter(sites.into_iter().enumerate())
        .map(|(idx, site)| {
            let state_ref = state;
            let app = app_handle.clone();
            let wd = wd.clone();
            async move {
                let group = search_site(state_ref, site, wd).await;
                let _ = app.emit(
                    "search:progress",
                    serde_json::json!({
                        "done": idx + 1,
                        "total": total,
                        "siteKey": group.site_key,
                        "siteName": group.site_name,
                        "ok": group.ok,
                        "count": group.items.len(),
                    }),
                );
                group
            }
        })
        .buffer_unordered(MAX_CONCURRENCY)
        .collect::<Vec<_>>()
        .await;

    Ok(results)
}
