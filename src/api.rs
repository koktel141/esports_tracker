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