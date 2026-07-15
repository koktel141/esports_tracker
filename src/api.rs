use crate::error::AppError;
use crate::models::{LiveMatch, ProMatch};

const PRO_MATCHES_URL: &str = "https://api.opendota.com/api/proMatches";
const LIVE_MATCHES_URL: &str = "https://api.opendota.com/api/live";

/// Fetches recently finished professional matches.
pub async fn fetch_pro_matches() -> Result<Vec<ProMatch>, AppError> {
    fetch_with_retry(PRO_MATCHES_URL, 3).await
}

/// Fetches matches currently in progress.
pub async fn fetch_live_matches() -> Result<Vec<LiveMatch>, AppError> {
    fetch_with_retry(LIVE_MATCHES_URL, 3).await
}

/// Generic GET + JSON decode with a small exponential backoff retry loop,
/// since the public OpenDota API occasionally rate-limits or times out.
async fn fetch_with_retry<T>(url: &str, max_attempts: u32) -> Result<Vec<T>, AppError>
where
    T: for<'de> serde::Deserialize<'de>,
{
    let mut last_err: Option<AppError> = None;

    for attempt in 0..max_attempts {
        if attempt > 0 {
            let backoff_ms = 300 * 2u64.pow(attempt - 1);
            tokio::time::sleep(std::time::Duration::from_millis(backoff_ms)).await;
        }

        match try_fetch::<T>(url).await {
            Ok(data) => return Ok(data),
            Err(e) => last_err = Some(e),
        }
    }

    Err(last_err.expect("at least one attempt is always made"))
}

async fn try_fetch<T>(url: &str) -> Result<Vec<T>, AppError>
where
    T: for<'de> serde::Deserialize<'de>,
{
    let response = reqwest::get(url).await?;

    if !response.status().is_success() {
        return Err(AppError::ServerError(response.status().as_u16()));
    }

    let data: Vec<T> = response.json().await?;
    Ok(data)
}

use crate::error::AppError;
use crate::models::ProMatch;

const API_URL: &str = "https://api.opendota.com/api/proMatches";

pub async fn fetch_pro_matches() -> Result<Vec<ProMatch>, AppError> {

    let response = reqwest::get(API_URL).await?;

    if !response.status().is_success() {
        return Err(AppError::ServerError(response.status().as_u16()));
    }

    let matches: Vec<ProMatch> = response.json().await?;
    Ok(matches)
}