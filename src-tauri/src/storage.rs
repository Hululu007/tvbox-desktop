// JSON 文件存储：subscriptions / favorites / history（app_data_dir 下）
use crate::error::AppResult;
use crate::models::{FavoriteItem, HistoryItem, Subscription};
use crate::state::RuntimeState;
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct SavedState {
    #[serde(default)]
    active_id: Option<String>,
    #[serde(default)]
    subscriptions: Vec<Subscription>,
}

fn subs_path(dir: &Path) -> PathBuf {
    dir.join("subscriptions.json")
}
fn fav_path(dir: &Path) -> PathBuf {
    dir.join("favorites.json")
}
fn hist_path(dir: &Path) -> PathBuf {
    dir.join("history.json")
}

fn read_json<T: serde::de::DeserializeOwned>(p: &Path) -> Option<T> {
    let text = std::fs::read_to_string(p).ok()?;
    serde_json::from_str(&text).ok()
}

fn write_json<T: serde::Serialize>(p: &Path, value: &T) -> AppResult<()> {
    if let Some(parent) = p.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let tmp = p.with_extension("json.tmp");
    std::fs::write(&tmp, serde_json::to_string_pretty(value).unwrap_or_default())?;
    std::fs::rename(&tmp, p)?;
    Ok(())
}

fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

pub fn gen_id() -> String {
    use std::sync::atomic::{AtomicU64, Ordering};
    static SEQ: AtomicU64 = AtomicU64::new(0);
    let seq = SEQ.fetch_add(1, Ordering::Relaxed);
    format!("sub-{}-{}", now_ms(), seq)
}

/// 首次启动写入内置订阅；加载全部持久化状态
pub fn load_all(dir: &Path) -> AppResult<RuntimeState> {
    let saved: Option<SavedState> = read_json(&subs_path(dir));
    let (subscriptions, active_id) = match saved {
        Some(s) if !s.subscriptions.is_empty() => (s.subscriptions, s.active_id),
        _ => {
            let cfg = crate::builtin::builtin_config();
            let subs = vec![Subscription {
                id: crate::builtin::BUILTIN_ID.to_string(),
                name: crate::builtin::BUILTIN_NAME.to_string(),
                url: crate::builtin::BUILTIN_ID.to_string(),
                created_at: now_ms(),
                updated_at: now_ms(),
                enabled: true,
                config: cfg,
            }];
            let rt = RuntimeState {
                subscriptions: subs,
                active_id: None,
            };
            save_all(dir, &rt)?;
            (rt.subscriptions, rt.active_id)
        }
    };
    let active_id = active_id.filter(|id| subscriptions.iter().any(|s| s.id == *id));
    Ok(RuntimeState {
        subscriptions,
        active_id,
    })
}

pub fn save_all(dir: &Path, rt: &RuntimeState) -> AppResult<()> {
    write_json(
        &subs_path(dir),
        &SavedState {
            active_id: rt.active_id.clone(),
            subscriptions: rt.subscriptions.clone(),
        },
    )
}

pub fn get_favorites(dir: &Path) -> AppResult<Vec<FavoriteItem>> {
    Ok(read_json(&fav_path(dir)).unwrap_or_default())
}

pub fn save_favorites(dir: &Path, items: &[FavoriteItem]) -> AppResult<()> {
    write_json(&fav_path(dir), &items)
}

pub fn get_history(dir: &Path) -> AppResult<Vec<HistoryItem>> {
    Ok(read_json(&hist_path(dir)).unwrap_or_default())
}

pub fn save_history(dir: &Path, items: &[HistoryItem]) -> AppResult<()> {
    write_json(&hist_path(dir), &items)
}

pub fn now() -> i64 {
    now_ms()
}
