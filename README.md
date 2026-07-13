# Esports Live Tracker 🎮

A terminal-based UI (TUI) application written in Rust that fetches and displays live professional Dota 2 matches using the OpenDota API.

## Features
* **Live Updates:** Automatically fetches new data asynchronously every 30 seconds.
(or u can change it in main.rs)
* **Modular Architecture:** Clean separation of concerns (API, Models, UI, Error Handling).
* **Robust Error Handling:** Custom `AppError` enum using `Result` and `Option` for safe data parsing.
* **Interactive TUI:** Built with `ratatui` and `crossterm` for a responsive, color-coded terminal interface.
* **Search Filter:** Ability to filter live matches by team name via command-line arguments.

## Tech Stack
* `tokio` (Async runtime)
* `reqwest` (HTTP Client)
* `serde` & `serde_json` (Data parsing)
* `ratatui` & `crossterm` (Terminal User Interface)

## How to Run
To run the dashboard with all live matches:
\`\`\`bash
cargo run
\`\`\`

To filter matches by a specific team name (e.g., NaVi):
\`\`\`bash
cargo run -- navi
\`\`\`

Press `q` or `Ctrl+C` to quit the application safely.