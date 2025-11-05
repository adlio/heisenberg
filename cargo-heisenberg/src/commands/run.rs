use anyhow::{Context, Result};
use crossterm::event::{self, Event, KeyCode};
use heisenberg::config::HeisenbergConfig;
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Terminal,
};
use std::io::{BufRead, BufReader};
use std::process::{Child, Command, Stdio};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

enum LogSource {
    Frontend(String),
    Backend(String),
}

pub fn run(cargo_args: Vec<String>) -> Result<()> {
    // Try to load config, or use smart defaults
    let spa = if let Ok(config) = HeisenbergConfig::from_file("heisenberg.toml") {
        config
            .spa
            .context("No default SPA configured in heisenberg.toml")?
    } else {
        // Smart defaults: infer from project structure
        infer_spa_config()?
    };

    // Start frontend dev server
    let dev_cmd = spa.dev_command.as_deref().unwrap_or("npm run dev");
    let parts: Vec<&str> = dev_cmd.split_whitespace().collect();

    let frontend = Command::new(parts[0])
        .args(&parts[1..])
        .current_dir(&spa.working_dir)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .context("Failed to start frontend dev server")?;

    // Start backend with env vars for proxy mode and to disable its own autostart
    let backend = Command::new("cargo")
        .arg("run")
        .args(&cargo_args)
        .env("HEISENBERG_MODE", "proxy")
        .env("HEISENBERG_AUTOSTART_ORIGIN", "false")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .context("Failed to start backend")?;

    // Run TUI
    run_tui(frontend, backend)?;

    Ok(())
}

fn run_tui(mut frontend: Child, mut backend: Child) -> Result<()> {
    let (tx, rx) = mpsc::channel();

    // Spawn log readers
    let tx_fe = tx.clone();
    if let Some(stdout) = frontend.stdout.take() {
        thread::spawn(move || {
            let reader = BufReader::new(stdout);
            for line in reader.lines().map_while(Result::ok) {
                let _ = tx_fe.send(LogSource::Frontend(line));
            }
        });
    }

    let tx_be = tx.clone();
    if let Some(stdout) = backend.stdout.take() {
        thread::spawn(move || {
            let reader = BufReader::new(stdout);
            for line in reader.lines().map_while(Result::ok) {
                let _ = tx_be.send(LogSource::Backend(line));
            }
        });
    }

    // Setup terminal
    crossterm::terminal::enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    crossterm::execute!(
        stdout,
        crossterm::terminal::EnterAlternateScreen,
        crossterm::event::EnableMouseCapture
    )?;
    let backend_term = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend_term)?;

    let mut frontend_logs = Vec::new();
    let mut backend_logs = Vec::new();

    loop {
        // Collect logs
        while let Ok(log) = rx.try_recv() {
            match log {
                LogSource::Frontend(line) => frontend_logs.push(line),
                LogSource::Backend(line) => backend_logs.push(line),
            }
        }

        // Draw UI
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Percentage(50),
                    Constraint::Percentage(50),
                    Constraint::Length(1),
                ])
                .split(f.size());

            let fe_text: Vec<Line> = frontend_logs
                .iter()
                .rev()
                .take(chunks[0].height as usize - 2)
                .rev()
                .map(|l| Line::from(Span::raw(l.clone())))
                .collect();

            let be_text: Vec<Line> = backend_logs
                .iter()
                .rev()
                .take(chunks[1].height as usize - 2)
                .rev()
                .map(|l| Line::from(Span::raw(l.clone())))
                .collect();

            let fe_block = Block::default()
                .title("Frontend (npm)")
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::Cyan));

            let be_block = Block::default()
                .title("Backend (cargo)")
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::Green));

            f.render_widget(Paragraph::new(fe_text).block(fe_block), chunks[0]);
            f.render_widget(Paragraph::new(be_text).block(be_block), chunks[1]);

            let help = Paragraph::new("Press 'q' or 'c' to exit")
                .style(Style::default().fg(Color::DarkGray));
            f.render_widget(help, chunks[2]);
        })?;

        // Handle input
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.code == KeyCode::Char('q') || key.code == KeyCode::Char('c') {
                    break;
                }
            }
        }

        // Check if processes exited
        if frontend.try_wait()?.is_some() || backend.try_wait()?.is_some() {
            break;
        }
    }

    // Cleanup
    let _ = frontend.kill();
    let _ = backend.kill();

    crossterm::terminal::disable_raw_mode()?;
    crossterm::execute!(
        terminal.backend_mut(),
        crossterm::terminal::LeaveAlternateScreen,
        crossterm::event::DisableMouseCapture
    )?;

    Ok(())
}

fn infer_spa_config() -> Result<heisenberg::config::SpaConfig> {
    use std::path::{Path, PathBuf};

    // Look for ./web or ./frontend
    let working_dir = if Path::new("./web/package.json").exists() {
        PathBuf::from("./web")
    } else if Path::new("./frontend/package.json").exists() {
        PathBuf::from("./frontend")
    } else {
        anyhow::bail!(
            "No frontend found. Create heisenberg.toml or add ./web/package.json or ./frontend/package.json"
        );
    };

    // Infer output directory
    let output_dir = ["build", "dist", ".next", ".svelte-kit/output"]
        .iter()
        .map(|d| working_dir.join(d))
        .find(|p| p.exists())
        .unwrap_or_else(|| working_dir.join("build"));

    // Use library's smart inference for dev command and port
    let inferred = heisenberg::utils::infer_from_build_dir(&output_dir)
        .unwrap_or_else(|_| heisenberg::utils::InferredConfig::default_for_dir(&output_dir));

    Ok(heisenberg::config::SpaConfig {
        working_dir,
        output_dir,
        dev_command: Some(inferred.dev_command.join(" ")),
        build_command: None,
        dev_server: Some(inferred.dev_url),
    })
}
