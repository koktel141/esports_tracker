use std::fmt;

#[derive(Debug)]
pub enum AppError {
    NetworkError(reqwest::Error),
    ParseError(serde_json::Error),
    ServerError(u16),
    DbError(sqlx::Error),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AppError::NetworkError(e) => write!(f, "Network error: {}", e),
            AppError::ParseError(e) => write!(f, "Data parsing error: {}", e),
            AppError::ServerError(code) => write!(f, "Server returned error code {}", code),
            AppError::DbError(e) => write!(f, "Database error: {}", e),
        }
    }
}

impl std::error::Error for AppError {}

impl From<reqwest::Error> for AppError {
    fn from(e: reqwest::Error) -> Self {
        AppError::NetworkError(e)
    }
}

impl From<serde_json::Error> for AppError {
    fn from(e: serde_json::Error) -> Self {
        AppError::ParseError(e)
    }
}

impl From<sqlx::Error> for AppError {
    fn from(e: sqlx::Error) -> Self {
        AppError::DbError(e)
    }
}
