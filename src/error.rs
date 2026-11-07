use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("网络请求失败: {0}")]
    Network(#[from] reqwest::Error),

    #[error("响应解析失败: {0}")]
    Json(#[from] serde_json::Error),

    #[error("输入输出错误: {0}")]
    Io(#[from] std::io::Error),

    #[error("输入错误: {0}")]
    InvalidInput(String),

    #[error("未找到词典条目: {0}")]
    NotFound(String),

    #[error("远端响应错误: {0}")]
    Protocol(String),
}

pub type Result<T> = std::result::Result<T, AppError>;
