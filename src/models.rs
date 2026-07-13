use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ProMatch {
    pub match_id: i64,
    pub duration: i64,
    pub start_time: i64,
    pub radiant_team_id: Option<i64>,
    pub radiant_name: Option<String>,
    pub dire_team_id: Option<i64>,
    pub dire_name: Option<String>,
    pub league_name: Option<String>,
    pub radiant_win: bool,
}