// 内置订阅：来自原项目 samples/public-sources.json（MIT 协议，注明出处）
pub const BUILTIN_PUBLIC_SOURCES_JSON: &str = include_str!("builtin/public-sources.json");

pub const BUILTIN_ID: &str = "builtin:public-sources";
pub const BUILTIN_NAME: &str = "内置推荐订阅（公开采集站）";

pub fn builtin_config() -> crate::models::TvboxConfig {
    crate::models::TvboxConfig::parse_json(BUILTIN_PUBLIC_SOURCES_JSON)
        .expect("builtin config must be valid")
}
