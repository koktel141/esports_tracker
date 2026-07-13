mod api;
mod error;
mod models;
mod ui;

use crossterm::{
    event::{Event, EventStream, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use futures::StreamExt;
use models::ProMatch;
use ratatui::{backend::CrosstermBackend, Terminal};
use std::env;
use std::io::stdout;
use std::time::Duration;
use tokio::time;

pub struct AppState {
    pub matches: Vec<ProMatch>,
    pub filter: Option<String>,
    pub status: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let filter = env::args().nth(1);

    // Terminal setup: raw mode disables line buffering so we get keys instantly,
    // alternate screen keeps the user's normal shell content intact underneath.
    enable_raw_mode()?;
    let mut out = stdout();
    execute!(out, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(out);
    let mut terminal = Terminal::new(backend)?;

    let mut state = AppState {
        matches: Vec::new(),
        filter,
        status: "Loading...".to_string(),
    };

    load_matches(&mut state).await;
    terminal.draw(|frame| ui::draw(frame, &state))?;

    let mut ticker = time::interval(Duration::from_secs(30));
    let mut events = EventStream::new();

    loop {
        tokio::select! {
            _ = ticker.tick() => {
                load_matches(&mut state).await;
                terminal.draw(|frame| ui::draw(frame, &state))?;
            }
            maybe_event = events.next() => {
                if let Some(Ok(Event::Key(key))) = maybe_event {
                    let is_quit = key.code == KeyCode::Char('q')
                        || (key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL));

                    if is_quit {
                        break;
                    }
                }
            }
        }
    }

    // Always restore the terminal, even on manual exit.
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    Ok(())
}

async fn load_matches(state: &mut AppState) {
    match api::fetch_pro_matches().await {
        Ok(matches) => {
            state.status = format!("Last update OK - {} matches", matches.len());
            state.matches = matches;
        }
        Err(e) => {
            state.status = format!("Fetch failed: {} (keeping old data)", e);
        }
    }
}