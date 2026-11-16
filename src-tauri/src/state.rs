// 应用全局状态：数据目录 + HTTP 客户端 + 订阅运行时状态
use crate::models::Subscription;
use std::path::PathBuf;
use tokio::sync::Mutex;

pub struct AppState {
    pub data_dir: PathBuf,
    pub client: reqwest::Client,
    pub runtime: Mutex<RuntimeState>,
}

#[derive(Default)]
pub struct RuntimeState {
    pub subscriptions: Vec<Subscription>,
    pub active_id: Option<String>,
}

impl AppState {
    pub fn active_config(&self, rt: &RuntimeState) -> Option<crate::models::TvboxConfig> {
        let active = rt
            .active_id
            .as_ref()
            .and_then(|id| rt.subscriptions.iter().find(|s| &s.id == id));
        // 若无激活订阅则回退到第一个
        let sub = match active {
            Some(s) => Some(s),
            None => rt.subscriptions.first(),
        };
        sub.map(|s| s.config.clone())
    }
}

pub const USER_AGENT: &str =
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0 Safari/537.36 tvbox-desktop-tauri/1.0";
