mod api;
mod app_state;
mod db;
mod error;
mod models;
mod ui;

use app_state::{AppState, InputMode, Tab};
use crossterm::{
    event::{Event, EventStream, KeyCode, KeyEventKind, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use futures::StreamExt;
use models::{LiveMatch, ProMatch, TeamStat};
use ratatui::{backend::CrosstermBackend, Terminal};
use sqlx::SqlitePool;
use std::env;
use std::io::stdout;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time;

/// Results reported back from background network/DB tasks. Handling these
/// in the select loop as messages (instead of awaiting the fetch directly
/// in the loop) keeps keyboard input responsive even when OpenDota is slow.
enum RefreshMsg {
    ProMatches {
        matches: Vec<ProMatch>,
        stats: Vec<TeamStat>,
    },
    ProMatchesFailed(String),
    LiveMatches(Vec<LiveMatch>),
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let filter = env::args().nth(1);

    let pool = db::init_db().await?;

    // Terminal setup: raw mode disables line buffering so we get keys instantly,
    // alternate screen keeps the user's normal shell content intact underneath.
    enable_raw_mode()?;
    let mut out = stdout();
    execute!(out, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(out);
    let mut terminal = Terminal::new(backend)?;

    let mut state = AppState::new(filter);

    // Seed from cache immediately so the UI has something to show even
    // before the first network round trip completes.
    if let Ok(cached) = db::load_cached_matches(&pool, 100).await {
        state.pro_matches = cached;
    }
    if let Ok(stats) = db::team_stats(&pool).await {
        state.team_stats = stats;
    }

    let (tx, mut rx) = mpsc::channel::<RefreshMsg>(8);

    // Kick off the first fetches in the background rather than awaiting
    // them here, so the UI is interactive immediately.
    spawn_pro_refresh(pool.clone(), tx.clone());
    spawn_live_refresh(tx.clone());

    terminal.draw(|frame| ui::draw(frame, &state))?;

    let mut pro_ticker = time::interval(Duration::from_secs(30));
    let mut live_ticker = time::interval(Duration::from_secs(10));
    let mut spinner_ticker = time::interval(Duration::from_millis(120));
    let mut events = EventStream::new();

    loop {
        tokio::select! {
            _ = pro_ticker.tick() => {
                state.is_loading_pro = true;
                spawn_pro_refresh(pool.clone(), tx.clone());
            }
            _ = live_ticker.tick() => {
                state.is_loading_live = true;
                spawn_live_refresh(tx.clone());
            }
            _ = spinner_ticker.tick() => {
                if state.is_loading() {
                    state.spinner_frame = state.spinner_frame.wrapping_add(1);
                    terminal.draw(|frame| ui::draw(frame, &state))?;
                }
            }
            msg = rx.recv() => {
                if let Some(msg) = msg {
                    apply_refresh(&mut state, msg);
                    terminal.draw(|frame| ui::draw(frame, &state))?;
                }
            }
            maybe_event = events.next() => {
                if let Some(Ok(Event::Key(key))) = maybe_event {
                    // Windows reports separate Press and Release events for every
                    // keystroke; only act on Press or we'd double-fire each action.
                    if key.kind != KeyEventKind::Press {
                        continue;
                    }
                    let should_quit = handle_key(key.code, key.modifiers, &mut state);
                    if should_quit {
                        break;
                    }
                    terminal.draw(|frame| ui::draw(frame, &state))?;
                }
            }
        }
    }

    // Always restore the terminal, even on manual exit.
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    Ok(())
}

/// Spawns a background task that fetches pro matches, updates the SQLite
/// cache, and recomputes team stats, then reports the outcome back through
/// `tx`. Runs off the main select loop so a slow API call never blocks
/// keyboard input.
fn spawn_pro_refresh(pool: SqlitePool, tx: mpsc::Sender<RefreshMsg>) {
    tokio::spawn(async move {
        let msg = match api::fetch_pro_matches().await {
            Ok(matches) => {
                if let Err(e) = db::upsert_matches(&pool, &matches).await {
                    let _ = tx
                        .send(RefreshMsg::ProMatchesFailed(format!(
                            "Cache write failed: {e}"
                        )))
                        .await;
                    return;
                }
                let stats = db::team_stats(&pool).await.unwrap_or_default();
                RefreshMsg::ProMatches { matches, stats }
            }
            Err(e) => RefreshMsg::ProMatchesFailed(format!(
                "Pro match fetch failed: {e} (showing cached data)"
            )),
        };
        let _ = tx.send(msg).await;
    });
}

fn spawn_live_refresh(tx: mpsc::Sender<RefreshMsg>) {
    tokio::spawn(async move {
        if let Ok(matches) = api::fetch_live_matches().await {
            let _ = tx.send(RefreshMsg::LiveMatches(matches)).await;
        }
    });
}

fn apply_refresh(state: &mut AppState, msg: RefreshMsg) {
    match msg {
        RefreshMsg::ProMatches { matches, stats } => {
            state.status = format!("Last update OK - {} pro matches cached", matches.len());
            state.pro_matches = matches;
            state.team_stats = stats;
            state.is_loading_pro = false;
        }
        RefreshMsg::ProMatchesFailed(reason) => {
            state.status = reason;
            state.is_loading_pro = false;
        }
        RefreshMsg::LiveMatches(matches) => {
            state.live_matches = matches;
            state.is_loading_live = false;
        }
    }
}

/// Returns true if the application should quit.
fn handle_key(code: KeyCode, modifiers: KeyModifiers, state: &mut AppState) -> bool {
    if state.input_mode == InputMode::EditingFilter {
        match code {
            KeyCode::Enter => state.confirm_filter_edit(),
            KeyCode::Esc => state.cancel_filter_edit(),
            KeyCode::Backspace => {
                state.filter_buffer.pop();
            }
            KeyCode::Char(c) => state.filter_buffer.push(c),
            _ => {}
        }
        return false;
    }

    match code {
        KeyCode::Char('q') => return true,
        KeyCode::Char('c') if modifiers.contains(KeyModifiers::CONTROL) => return true,
        _ => {}
    }

    if state.is_repeat(code) {
        return false;
    }

    match code {
        KeyCode::Tab => state.switch_tab(),
        KeyCode::Down => state.move_selection(1),
        KeyCode::Up => state.move_selection(-1),
        KeyCode::Enter => {
            if state.active_tab != Tab::Stats {
                state.show_detail = !state.show_detail;
            }
        }
        KeyCode::Esc => state.show_detail = false,
        KeyCode::Char('/') => state.start_filter_edit(),
        KeyCode::Char('s') => {
            if state.active_tab == Tab::Pro {
                state.sort_mode = state.sort_mode.next();
            }
        }
        _ => {}
    }

    false
}
