// 统一错误类型：跨 FFI 序列化为字符串，前端直接展示
use serde::Serialize;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("{0}")]
    Message(String),
    #[error("网络请求失败: {0}")]
    Http(String),
    #[error("配置解析失败: {0}")]
    Parse(String),
    #[error("存储读写失败: {0}")]
    Io(#[from] std::io::Error),
}

impl AppError {
    pub fn msg(s: impl Into<String>) -> Self {
        AppError::Message(s.into())
    }
}

impl Serialize for AppError {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

pub type AppResult<T> = Result<T, AppError>;
