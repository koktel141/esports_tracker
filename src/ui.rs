use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Paragraph, Row, Table},
    Frame,
};

use crate::app_state::{AppState, InputMode, Tab};

/// Consistent rounded-border block used across every panel, so the whole
/// UI shares one visual language instead of ratatui's default sharp corners.
fn rounded_block(title: impl Into<String>) -> Block<'static> {
    Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Rgb(90, 90, 120)))
        .title(Span::styled(
            title.into(),
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ))
}

pub fn draw(frame: &mut Frame, state: &AppState) {
    let area = frame.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(area);

    draw_tab_bar(frame, chunks[0], state);

    match state.active_tab {
        Tab::Pro => draw_pro_table(frame, chunks[1], state),
        Tab::Live => draw_live_table(frame, chunks[1], state),
        Tab::Stats => draw_stats_table(frame, chunks[1], state),
    }

    draw_help_bar(frame, chunks[2], state);

    if state.show_detail {
        draw_detail_popup(frame, area, state);
    }
}

fn draw_tab_bar(frame: &mut Frame, area: Rect, state: &AppState) {
    let titles = [Tab::Pro, Tab::Live, Tab::Stats];
    let mut spans = Vec::new();
    for (i, tab) in titles.iter().enumerate() {
        let icon = match tab {
            Tab::Pro => "🏆",
            Tab::Live => "⚡",
            Tab::Stats => "📊",
        };
        let style = if *tab == state.active_tab {
            Style::default()
                .fg(Color::Black)
                .bg(Color::Cyan)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::DarkGray)
        };
        spans.push(Span::styled(format!(" {icon} {} ", tab.label()), style));
        if i < titles.len() - 1 {
            spans.push(Span::raw("  "));
        }
    }

    if state.is_loading() {
        spans.push(Span::styled(
            format!("  {} syncing...", state.spinner_char()),
            Style::default().fg(Color::Yellow),
        ));
    }

    if let Some(f) = &state.filter {
        spans.push(Span::styled(
            format!("  🔎 {f}"),
            Style::default().fg(Color::LightMagenta),
        ));
    }

    let block = rounded_block(format!(" {} ", state.status));
    let paragraph = Paragraph::new(Line::from(spans)).block(block);
    frame.render_widget(paragraph, area);
}

fn draw_pro_table(frame: &mut Frame, area: Rect, state: &AppState) {
    let matches = state.visible_pro_matches();

    let rows: Vec<Row> = matches
        .iter()
        .enumerate()
        .map(|(i, m)| {
            let (icon, row_color) = if m.radiant_win {
                ("🟢", Color::Green)
            } else {
                ("🔴", Color::Red)
            };
            let mut style = Style::default().fg(row_color);
            if i == state.selected_index {
                style = style.bg(Color::Rgb(40, 40, 60)).add_modifier(Modifier::BOLD);
            }
            Row::new(vec![
                format!("{icon} {}", m.match_id),
                m.radiant_label().to_string(),
                m.dire_label().to_string(),
                m.winner_label().to_string(),
                m.duration_label(),
            ])
            .style(style)
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Length(16),
            Constraint::Percentage(28),
            Constraint::Percentage(28),
            Constraint::Percentage(20),
            Constraint::Length(10),
        ],
    )
    .header(
        Row::new(vec!["Match ID", "Radiant", "Dire", "Winner", "Duration"])
            .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
    )
    .block(rounded_block(format!(
        " Pro Matches — sort: {} ",
        state.sort_mode.label()
    )));

    frame.render_widget(table, area);
}

fn draw_live_table(frame: &mut Frame, area: Rect, state: &AppState) {
    let matches = state.visible_live_matches();

    let rows: Vec<Row> = matches
        .iter()
        .enumerate()
        .map(|(i, m)| {
            let mut style = Style::default().fg(Color::LightCyan);
            if i == state.selected_index {
                style = style.bg(Color::Rgb(40, 40, 60)).add_modifier(Modifier::BOLD);
            }
            Row::new(vec![
                format!("⚡ {}", m.match_id),
                m.radiant_label().to_string(),
                m.dire_label().to_string(),
                m.score_label(),
                m.game_time_label(),
                m.spectators.map(|s| s.to_string()).unwrap_or_default(),
            ])
            .style(style)
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Length(16),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Length(10),
            Constraint::Length(10),
            Constraint::Length(12),
        ],
    )
    .header(
        Row::new(vec![
            "Match ID", "Radiant", "Dire", "Score", "Time", "Viewers",
        ])
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
    )
    .block(rounded_block(" Live Matches "));

    frame.render_widget(table, area);
}

fn draw_stats_table(frame: &mut Frame, area: Rect, state: &AppState) {
    let rows: Vec<Row> = state
        .team_stats
        .iter()
        .enumerate()
        .map(|(i, s)| {
            let rank_icon = match i {
                0 => "🥇",
                1 => "🥈",
                2 => "🥉",
                _ => "  ",
            };
            let mut style = Style::default().fg(Color::White);
            if i == state.selected_index {
                style = style.bg(Color::Rgb(40, 40, 60)).add_modifier(Modifier::BOLD);
            }
            Row::new(vec![
                format!("{rank_icon} {}", s.team_name),
                s.wins.to_string(),
                s.losses.to_string(),
                format!("{:.1}%", s.win_rate()),
            ])
            .style(style)
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Percentage(40),
            Constraint::Length(10),
            Constraint::Length(10),
            Constraint::Length(12),
        ],
    )
    .header(
        Row::new(vec!["Team", "Wins", "Losses", "Win rate"])
            .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
    )
    .block(rounded_block(" Team Stats — from cached match history "));

    frame.render_widget(table, area);
}

fn draw_help_bar(frame: &mut Frame, area: Rect, state: &AppState) {
    let text = match state.input_mode {
        InputMode::EditingFilter => format!(
            " 🔎 Filter: {}▎  (Enter=confirm, Esc=cancel)",
            state.filter_buffer
        ),
        InputMode::Normal => " ⭾ switch view   ↑/↓ select   ⏎ details   / filter   s sort   q quit"
            .to_string(),
    };
    let style = if state.input_mode == InputMode::EditingFilter {
        Style::default()
            .fg(Color::Black)
            .bg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::DarkGray)
    };
    frame.render_widget(Paragraph::new(text).style(style), area);
}

fn draw_detail_popup(frame: &mut Frame, area: Rect, state: &AppState) {
    let popup_area = centered_rect(60, 45, area);
    frame.render_widget(Clear, popup_area);

    let label_style = Style::default().fg(Color::DarkGray);
    let value_style = Style::default().fg(Color::White).add_modifier(Modifier::BOLD);
    let row = |label: &'static str, value: String| {
        Line::from(vec![
            Span::styled(format!("{label:<16}"), label_style),
            Span::styled(value, value_style),
        ])
    };

    let lines: Vec<Line> = match state.active_tab {
        Tab::Pro => {
            let matches = state.visible_pro_matches();
            match matches.get(state.selected_index) {
                Some(m) => {
                    let winner_color = if m.radiant_win { Color::Green } else { Color::Red };
                    vec![
                        row("Match ID", m.match_id.to_string()),
                        row("League", m.league_name.as_deref().unwrap_or("Unknown").to_string()),
                        Line::from(""),
                        row("Radiant", m.radiant_label().to_string()),
                        row("Dire", m.dire_label().to_string()),
                        Line::from(vec![
                            Span::styled(format!("{:<16}", "Winner"), label_style),
                            Span::styled(
                                format!("🏆 {}", m.winner_label()),
                                Style::default().fg(winner_color).add_modifier(Modifier::BOLD),
                            ),
                        ]),
                        Line::from(""),
                        row("Duration", m.duration_label()),
                        row(
                            "Radiant team id",
                            m.radiant_team_id.map(|id| id.to_string()).unwrap_or_else(|| "-".to_string()),
                        ),
                        row(
                            "Dire team id",
                            m.dire_team_id.map(|id| id.to_string()).unwrap_or_else(|| "-".to_string()),
                        ),
                    ]
                }
                None => vec![Line::from("No match selected")],
            }
        }
        Tab::Live => {
            let matches = state.visible_live_matches();
            match matches.get(state.selected_index) {
                Some(m) => vec![
                    row("Match ID", m.match_id.to_string()),
                    Line::from(""),
                    row("Radiant", m.radiant_label().to_string()),
                    row("Dire", m.dire_label().to_string()),
                    row("Score", m.score_label()),
                    Line::from(""),
                    row("Game time", m.game_time_label()),
                    row(
                        "Spectators",
                        m.spectators.map(|s| s.to_string()).unwrap_or_else(|| "-".to_string()),
                    ),
                    row(
                        "League id",
                        m.league_id.map(|id| id.to_string()).unwrap_or_else(|| "-".to_string()),
                    ),
                ],
                None => vec![Line::from("No match selected")],
            }
        }
        Tab::Stats => vec![],
    };

    let block = rounded_block(" 📋 Match Details  (Esc to close) ");
    let paragraph = Paragraph::new(lines).block(block).alignment(Alignment::Left);
    frame.render_widget(paragraph, popup_area);
}

/// Helper to compute a centered rectangle for popups, as a percentage of the parent area.
fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(vertical[1])[1]
}