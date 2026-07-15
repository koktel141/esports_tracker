use std::fmt;

#[derive(Debug)]
pub enum AppError {
    NetworkError(reqwest::Error),
    ParseError(serde_json::Error),
    ServerError(u16),
<<<<<<< HEAD
=======
    DbError(sqlx::Error),
>>>>>>> 126e2a1 (esp_live: multi-tab TUI dota tracker with SQLite caching)
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AppError::NetworkError(e) => write!(f, "Network error: {}", e),
            AppError::ParseError(e) => write!(f, "Data parsing error: {}", e),
            AppError::ServerError(code) => write!(f, "Server returned error code {}", code),
<<<<<<< HEAD
=======
            AppError::DbError(e) => write!(f, "Database error: {}", e),
>>>>>>> 126e2a1 (esp_live: multi-tab TUI dota tracker with SQLite caching)
        }
    }
}

<<<<<<< HEAD
=======
impl std::error::Error for AppError {}

>>>>>>> 126e2a1 (esp_live: multi-tab TUI dota tracker with SQLite caching)
impl From<reqwest::Error> for AppError {
    fn from(e: reqwest::Error) -> Self {
        AppError::NetworkError(e)
    }
}

impl From<serde_json::Error> for AppError {
    fn from(e: serde_json::Error) -> Self {
        AppError::ParseError(e)
    }
<<<<<<< HEAD
}
=======
}

impl From<sqlx::Error> for AppError {
    fn from(e: sqlx::Error) -> Self {
        AppError::DbError(e)
    }
}
>>>>>>> 126e2a1 (esp_live: multi-tab TUI dota tracker with SQLite caching)
