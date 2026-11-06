use serde::Serialize;

#[derive(Debug, thiserror::Error, Serialize)]
pub enum TitanError {
    #[error("[NETWORK] {0}")]
    Network(String),
    #[error("[BROKER] {0}")]
    Broker(String),
    #[error("[VALIDATION] {0}")]
    Validation(String),
    #[error("[INTERNAL] {0}")]
    Internal(String),
    #[error("[IO] {0}")]
    Io(String),
    #[error("[GPU] {0}")]
    Gpu(String),
    #[error("[TIMEOUT] {0}")]
    Timeout(String),
    #[error("[RATE_LIMIT] {0}")]
    RateLimit(String),
}

impl From<TitanError> for String {
    fn from(e: TitanError) -> String {
        e.to_string()
    }
}

impl From<reqwest::Error> for TitanError {
    fn from(e: reqwest::Error) -> Self {
        if e.is_timeout() {
            TitanError::Timeout(e.to_string())
        } else {
            TitanError::Network(e.to_string())
        }
    }
}

impl From<serde_json::Error> for TitanError {
    fn from(e: serde_json::Error) -> Self {
        TitanError::Internal(format!("JSON: {}", e))
    }
}

impl From<std::io::Error> for TitanError {
    fn from(e: std::io::Error) -> Self {
        TitanError::Io(e.to_string())
    }
}

impl From<toml::de::Error> for TitanError {
    fn from(e: toml::de::Error) -> Self {
        TitanError::Internal(format!("TOML: {}", e))
    }
}

impl From<String> for TitanError {
    fn from(s: String) -> Self {
        TitanError::Internal(s)
    }
}

#[macro_export]
macro_rules! titan_bail {
    ($variant:ident, $($arg:tt)*) => {
        return Err($crate::error::TitanError::$variant(format!($($arg)*)))
    };
}
