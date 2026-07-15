use serde::Deserialize;

<<<<<<< HEAD
#[derive(Debug, Deserialize)]
=======
#[derive(Debug, Clone, Deserialize)]
>>>>>>> 126e2a1 (esp_live: multi-tab TUI dota tracker with SQLite caching)
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
<<<<<<< HEAD
}
=======
}

impl ProMatch {
    pub fn radiant_label(&self) -> &str {
        self.radiant_name.as_deref().unwrap_or("Radiant")
    }

    pub fn dire_label(&self) -> &str {
        self.dire_name.as_deref().unwrap_or("Dire")
    }

    pub fn winner_label(&self) -> &str {
        if self.radiant_win {
            self.radiant_label()
        } else {
            self.dire_label()
        }
    }

    pub fn duration_label(&self) -> String {
        let mins = self.duration / 60;
        let secs = self.duration % 60;
        format!("{mins}:{secs:02}")
    }
}

/// Live match data from OpenDota's `/live` endpoint. Field availability
/// varies match-to-match, so almost everything is optional.
#[derive(Debug, Clone, Deserialize)]
pub struct LiveMatch {
    pub match_id: i64,
    pub radiant_name: Option<String>,
    pub dire_name: Option<String>,
    #[serde(default)]
    pub radiant_score: Option<i64>,
    #[serde(default)]
    pub dire_score: Option<i64>,
    #[serde(default)]
    pub spectators: Option<i64>,
    #[serde(default)]
    pub game_time: Option<i64>,
    #[serde(default)]
    pub league_id: Option<i64>,
}

impl LiveMatch {
    pub fn radiant_label(&self) -> &str {
        self.radiant_name.as_deref().unwrap_or("Radiant")
    }

    pub fn dire_label(&self) -> &str {
        self.dire_name.as_deref().unwrap_or("Dire")
    }

    pub fn score_label(&self) -> String {
        format!(
            "{} - {}",
            self.radiant_score.unwrap_or(0),
            self.dire_score.unwrap_or(0)
        )
    }

    pub fn game_time_label(&self) -> String {
        match self.game_time {
            Some(t) if t >= 0 => {
                let mins = t / 60;
                let secs = t % 60;
                format!("{mins}:{secs:02}")
            }
            _ => "--:--".to_string(),
        }
    }
}

/// Aggregated win/loss record for a team, computed from cached match history.
#[derive(Debug, Clone)]
pub struct TeamStat {
    pub team_name: String,
    pub wins: i64,
    pub losses: i64,
}

impl TeamStat {
    pub fn total(&self) -> i64 {
        self.wins + self.losses
    }

    pub fn win_rate(&self) -> f64 {
        if self.total() == 0 {
            0.0
        } else {
            (self.wins as f64 / self.total() as f64) * 100.0
        }
    }
}
>>>>>>> 126e2a1 (esp_live: multi-tab TUI dota tracker with SQLite caching)
