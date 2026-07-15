use crate::models::{LiveMatch, ProMatch, TeamStat};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Pro,
    Live,
    Stats,
}

impl Tab {
    pub fn label(&self) -> &'static str {
        match self {
            Tab::Pro => "Pro Matches",
            Tab::Live => "Live Matches",
            Tab::Stats => "Team Stats",
        }
    }

    pub fn next(self) -> Self {
        match self {
            Tab::Pro => Tab::Live,
            Tab::Live => Tab::Stats,
            Tab::Stats => Tab::Pro,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortMode {
    Newest,
    Duration,
    Winner,
}

impl SortMode {
    pub fn next(self) -> Self {
        match self {
            SortMode::Newest => SortMode::Duration,
            SortMode::Duration => SortMode::Winner,
            SortMode::Winner => SortMode::Newest,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            SortMode::Newest => "newest first",
            SortMode::Duration => "duration",
            SortMode::Winner => "winner name",
        }
    }
}

/// Whether the user is currently typing into the filter box.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputMode {
    Normal,
    EditingFilter,
}

pub struct AppState {
    pub pro_matches: Vec<ProMatch>,
    pub live_matches: Vec<LiveMatch>,
    pub team_stats: Vec<TeamStat>,

    pub active_tab: Tab,
    pub sort_mode: SortMode,
    pub filter: Option<String>,
    pub filter_buffer: String,
    pub input_mode: InputMode,

    pub selected_index: usize,
    pub show_detail: bool,

    pub status: String,

    /// Whether a pro/live refresh is currently in flight, so the UI can
    /// show an animated spinner instead of sitting silently.
    pub is_loading_pro: bool,
    pub is_loading_live: bool,
    pub spinner_frame: usize,

    /// Tracks the most recently processed key + when, so terminal key-repeat
    /// (holding a key down, or duplicate press/release events some terminals
    /// send for a single physical keystroke) doesn't trigger the action
    /// multiple times in a row.
    last_key: Option<crossterm::event::KeyCode>,
    last_key_time: std::time::Instant,
}

impl AppState {
    pub fn new(initial_filter: Option<String>) -> Self {
        Self {
            pro_matches: Vec::new(),
            live_matches: Vec::new(),
            team_stats: Vec::new(),
            active_tab: Tab::Pro,
            sort_mode: SortMode::Newest,
            filter: initial_filter,
            filter_buffer: String::new(),
            input_mode: InputMode::Normal,
            selected_index: 0,
            show_detail: false,
            status: "Loading...".to_string(),
            is_loading_pro: true,
            is_loading_live: true,
            spinner_frame: 0,
            last_key: None,
            last_key_time: std::time::Instant::now(),
        }
    }

    pub fn visible_pro_matches(&self) -> Vec<&ProMatch> {
        let mut matches: Vec<&ProMatch> = self
            .pro_matches
            .iter()
            .filter(|m| self.matches_filter(m.radiant_label(), m.dire_label()))
            .collect();

        match self.sort_mode {
            SortMode::Newest => matches.sort_by(|a, b| b.start_time.cmp(&a.start_time)),
            SortMode::Duration => matches.sort_by(|a, b| b.duration.cmp(&a.duration)),
            SortMode::Winner => matches.sort_by(|a, b| a.winner_label().cmp(b.winner_label())),
        }

        matches
    }

    pub fn visible_live_matches(&self) -> Vec<&LiveMatch> {
        self.live_matches
            .iter()
            .filter(|m| self.matches_filter(m.radiant_label(), m.dire_label()))
            .collect()
    }

    fn matches_filter(&self, radiant: &str, dire: &str) -> bool {
        match &self.filter {
            None => true,
            Some(name) => {
                let name_lower = name.to_lowercase();
                radiant.to_lowercase().contains(&name_lower)
                    || dire.to_lowercase().contains(&name_lower)
            }
        }
    }

    pub fn current_list_len(&self) -> usize {
        match self.active_tab {
            Tab::Pro => self.visible_pro_matches().len(),
            Tab::Live => self.visible_live_matches().len(),
            Tab::Stats => self.team_stats.len(),
        }
    }

    pub fn move_selection(&mut self, delta: i32) {
        let len = self.current_list_len();
        if len == 0 {
            self.selected_index = 0;
            return;
        }
        let current = self.selected_index as i32;
        let new_index = (current + delta).rem_euclid(len as i32);
        self.selected_index = new_index as usize;
    }

    /// How long the *same* key must be quiet before it's accepted again.
    /// Filters out terminal key-repeat and duplicate press/release events
    /// from a single physical keystroke, without needing special terminal
    /// support to detect real key-up events.
    const KEY_DEBOUNCE: std::time::Duration = std::time::Duration::from_millis(180);

    /// Returns true if this keycode should be ignored because it's a
    /// repeat/duplicate of the same key seen very recently. Call this once
    /// per raw key event, before running any action for that key.
    pub fn is_repeat(&mut self, code: crossterm::event::KeyCode) -> bool {
        let now = std::time::Instant::now();
        let is_repeat = self.last_key == Some(code)
            && now.duration_since(self.last_key_time) < Self::KEY_DEBOUNCE;

        self.last_key = Some(code);
        self.last_key_time = now;

        is_repeat
    }

    pub fn switch_tab(&mut self) {
        self.active_tab = self.active_tab.next();
        self.selected_index = 0;
        self.show_detail = false;
    }

    pub fn start_filter_edit(&mut self) {
        self.input_mode = InputMode::EditingFilter;
        self.filter_buffer = self.filter.clone().unwrap_or_default();
    }

    pub fn confirm_filter_edit(&mut self) {
        self.filter = if self.filter_buffer.trim().is_empty() {
            None
        } else {
            Some(self.filter_buffer.trim().to_string())
        };
        self.input_mode = InputMode::Normal;
        self.selected_index = 0;
    }

    pub fn cancel_filter_edit(&mut self) {
        self.input_mode = InputMode::Normal;
        self.filter_buffer.clear();
    }

    const SPINNER_FRAMES: [&'static str; 10] =
        ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

    pub fn is_loading(&self) -> bool {
        self.is_loading_pro || self.is_loading_live
    }

    pub fn spinner_char(&self) -> &'static str {
        Self::SPINNER_FRAMES[self.spinner_frame % Self::SPINNER_FRAMES.len()]
    }
}