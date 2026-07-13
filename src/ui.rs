use crate::models::ProMatch;
use crate::AppState;
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Row, Table},
    Frame,
};

pub fn draw(frame: &mut Frame, state: &AppState) {
    let area = frame.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(area);

    let title = match &state.filter {
        Some(team) => format!("Esports Live Tracker - filter: {}", team),
        None => "Esports Live Tracker - all matches".to_string(),
    };

    let header = Paragraph::new(Line::from(vec![Span::styled(
        title,
        Style::default().add_modifier(Modifier::BOLD),
    )]))
    .block(Block::default().borders(Borders::ALL).title(state.status.as_str()));

    frame.render_widget(header, chunks[0]);

    let filtered = filter_matches(&state.matches, state.filter.as_deref());

    let rows: Vec<Row> = filtered
        .iter()
        .take(20)
        .map(|m| {
            let radiant = m.radiant_name.as_deref().unwrap_or("Radiant");
            let dire = m.dire_name.as_deref().unwrap_or("Dire");
            let winner = if m.radiant_win { radiant } else { dire };
            let row_color = if m.radiant_win {
                Color::Green  
}           else {
                Color::Red    
};
            Row::new(vec![
                m.match_id.to_string(),
                radiant.to_string(),
                dire.to_string(),
                winner.to_string(),
            ])
            .style(Style::default().fg(row_color))
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Length(14),
            Constraint::Percentage(30),
            Constraint::Percentage(30),
            Constraint::Percentage(25),
        ],
    )
    .header(
        Row::new(vec!["Match ID", "Radiant", "Dire", "Winner"])
            .style(Style::default().add_modifier(Modifier::BOLD)),
    )
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title("Matches (press q to quit)"),
    );

    frame.render_widget(table, chunks[1]);
}

fn filter_matches<'a>(matches: &'a [ProMatch], team: Option<&str>) -> Vec<&'a ProMatch> {
    match team {
        None => matches.iter().collect(),
        Some(name) => {
            let name_lower = name.to_lowercase();
            matches
                .iter()
                .filter(|m| {
                    let radiant_match = m
                        .radiant_name
                        .as_deref()
                        .map(|n| n.to_lowercase().contains(&name_lower))
                        .unwrap_or(false);
                    let dire_match = m
                        .dire_name
                        .as_deref()
                        .map(|n| n.to_lowercase().contains(&name_lower))
                        .unwrap_or(false);
                    radiant_match || dire_match
                })
                .collect()
        }
    }
}