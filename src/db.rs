use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::{Row, SqlitePool};
use std::str::FromStr;

use crate::error::AppError;
use crate::models::{ProMatch, TeamStat};

const DB_FILE: &str = "esp_live.db";

pub async fn init_db() -> Result<SqlitePool, AppError> {
    let options = SqliteConnectOptions::from_str(&format!("sqlite://{DB_FILE}"))?
        .create_if_missing(true);

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(options)
        .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS matches (
            match_id        INTEGER PRIMARY KEY,
            duration        INTEGER NOT NULL,
            start_time      INTEGER NOT NULL,
            radiant_team_id INTEGER,
            radiant_name    TEXT,
            dire_team_id    INTEGER,
            dire_name       TEXT,
            league_name     TEXT,
            radiant_win     INTEGER NOT NULL
        )
        "#,
    )
    .execute(&pool)
    .await?;

    Ok(pool)
}

/// Inserts new matches and overwrites any that already exist (e.g. if
/// OpenDota amends a match record after the fact).
pub async fn upsert_matches(pool: &SqlitePool, matches: &[ProMatch]) -> Result<(), AppError> {
    for m in matches {
        sqlx::query(
            r#"
            INSERT INTO matches
                (match_id, duration, start_time, radiant_team_id, radiant_name,
                 dire_team_id, dire_name, league_name, radiant_win)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(match_id) DO UPDATE SET
                duration = excluded.duration,
                start_time = excluded.start_time,
                radiant_team_id = excluded.radiant_team_id,
                radiant_name = excluded.radiant_name,
                dire_team_id = excluded.dire_team_id,
                dire_name = excluded.dire_name,
                league_name = excluded.league_name,
                radiant_win = excluded.radiant_win
            "#,
        )
        .bind(m.match_id)
        .bind(m.duration)
        .bind(m.start_time)
        .bind(m.radiant_team_id)
        .bind(&m.radiant_name)
        .bind(m.dire_team_id)
        .bind(&m.dire_name)
        .bind(&m.league_name)
        .bind(m.radiant_win)
        .execute(pool)
        .await?;
    }

    Ok(())
}

/// Loads the most recent cached matches, newest first. Used as a fallback
/// when the live API call fails, and to seed the Pro Matches tab on launch.
pub async fn load_cached_matches(pool: &SqlitePool, limit: i64) -> Result<Vec<ProMatch>, AppError> {
    let rows = sqlx::query(
        r#"
        SELECT match_id, duration, start_time, radiant_team_id, radiant_name,
        dire_team_id, dire_name, league_name, radiant_win
        FROM matches
        ORDER BY start_time DESC
        LIMIT ?
        "#,
    )
    .bind(limit)
    .fetch_all(pool)
    .await?;

    let matches = rows
        .into_iter()
        .map(|row| ProMatch {
            match_id: row.get("match_id"),
            duration: row.get("duration"),
            start_time: row.get("start_time"),
            radiant_team_id: row.get("radiant_team_id"),
            radiant_name: row.get("radiant_name"),
            dire_team_id: row.get("dire_team_id"),
            dire_name: row.get("dire_name"),
            league_name: row.get("league_name"),
            radiant_win: row.get("radiant_win"),
        })
        .collect();

    Ok(matches)
}

/// Computes aggregated win/loss records per team from all cached matches.
/// A team's record combines its results whether it played as Radiant or Dire.
pub async fn team_stats(pool: &SqlitePool) -> Result<Vec<TeamStat>, AppError> {
    let rows = sqlx::query(
        r#"
        WITH team_results AS (
            SELECT radiant_name AS team_name, radiant_win AS won
            FROM matches
            WHERE radiant_name IS NOT NULL
            UNION ALL
            SELECT dire_name AS team_name, (1 - radiant_win) AS won
            FROM matches
            WHERE dire_name IS NOT NULL
        )
        SELECT
            team_name,
            SUM(won) AS wins,
            SUM(1 - won) AS losses
        FROM team_results
        GROUP BY team_name
        ORDER BY wins DESC
        "#,
    )
    .fetch_all(pool)
    .await?;

    let stats = rows
        .into_iter()
        .map(|row| TeamStat {
            team_name: row.get("team_name"),
            wins: row.get("wins"),
            losses: row.get("losses"),
        })
        .collect();

    Ok(stats)
}
