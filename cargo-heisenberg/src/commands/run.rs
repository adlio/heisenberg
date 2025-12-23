use anyhow::{Context, Result};
use crossterm::event::{self, Event, KeyCode};
use heisenberg::config::{HeisenbergConfig, SpaConfig};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Terminal,
};
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use super::infer;

enum LogSource {
    Frontend(usize, String), // (spa_index, line)
    Backend(String),
}

/// Parsed run command arguments.
#[derive(Debug, Clone, PartialEq)]
pub struct RunArgs {
    /// Whether to run in plain mode (no TUI)
    pub no_tui: bool,
    /// Arguments to pass to cargo run
    pub cargo_args: Vec<String>,
}

impl RunArgs {
    /// Parse cargo arguments, extracting --no-tui flag.
    pub fn parse(cargo_args: Vec<String>) -> Self {
        let no_tui = cargo_args.iter().any(|arg| arg == "--no-tui");
        let cargo_args: Vec<String> = cargo_args
            .into_iter()
            .filter(|arg| arg != "--no-tui")
            .collect();

        Self { no_tui, cargo_args }
    }
}

/// Run the dev server command in the current directory.
pub fn run(cargo_args: Vec<String>) -> Result<()> {
    run_in_dir(Path::new("."), cargo_args)
}

/// Run the dev server command in the specified directory.
pub fn run_in_dir(base_dir: &Path, cargo_args: Vec<String>) -> Result<()> {
    let args = RunArgs::parse(cargo_args);
    let config_path = base_dir.join("heisenberg.toml");

    // Try to load config, or use smart defaults
    let spas = if let Ok(config) = HeisenbergConfig::from_file(&config_path) {
        let spa_configs = config.spas();
        if spa_configs.is_empty() {
            anyhow::bail!("No SPA configurations found in heisenberg.toml");
        }
        spa_configs.into_iter().cloned().collect::<Vec<_>>()
    } else {
        // Smart defaults: infer from project structure
        vec![infer_spa_config(base_dir)?]
    };

    // Start all frontend dev servers
    let mut frontends = Vec::new();
    for (idx, spa) in spas.iter().enumerate() {
        let working_dir = base_dir.join(&spa.working_dir);
        let node_modules = working_dir.join("node_modules");
        let package_json = working_dir.join("package.json");
        let dev_cmd = spa.dev_command.as_deref().unwrap_or("npm run dev");

        // Check if npm install is needed
        let needs_install = check_needs_install(&node_modules, &package_json);

        let full_cmd = if needs_install {
            format!("npm install && {}", dev_cmd)
        } else {
            dev_cmd.to_string()
        };

        let frontend = Command::new("sh")
            .arg("-c")
            .arg(&full_cmd)
            .current_dir(&working_dir)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .with_context(|| format!("Failed to start frontend {} at {:?}", idx, working_dir))?;

        frontends.push((idx, frontend, working_dir.display().to_string()));
    }

    // Start backend with env vars for proxy mode
    let backend = Command::new("cargo")
        .arg("run")
        .args(&args.cargo_args)
        .current_dir(base_dir)
        .env("HEISENBERG_MODE", "proxy")
        .env("HEISENBERG_AUTOSTART_ORIGIN", "false")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .context("Failed to start backend")?;

    // Run TUI or plain mode
    if args.no_tui {
        run_plain(frontends, backend)?;
    } else {
        run_tui(frontends, backend)?;
    }

    Ok(())
}

/// Check if npm install is needed based on node_modules existence and timestamps.
fn check_needs_install(node_modules: &Path, package_json: &Path) -> bool {
    if !node_modules.exists() {
        return true;
    }

    // Check if package.json is newer than node_modules
    if let (Ok(pkg_meta), Ok(nm_meta)) = (
        std::fs::metadata(package_json),
        std::fs::metadata(node_modules),
    ) {
        if let (Ok(pkg_time), Ok(nm_time)) = (pkg_meta.modified(), nm_meta.modified()) {
            return pkg_time > nm_time;
        }
    }

    false
}

fn run_plain(mut frontends: Vec<(usize, Child, String)>, mut backend: Child) -> Result<()> {
    use std::io::Write;

    println!("🚀 Running in plain mode (logs to stdout/stderr)");
    println!("   Press Ctrl+C to exit\n");

    // Just wait for processes and let their output go to stdout/stderr naturally
    let (tx, rx) = mpsc::channel();

    // Spawn threads to monitor process exit
    for (idx, child, path) in frontends.iter_mut() {
        let idx = *idx;
        let path = path.clone();
        let tx_clone = tx.clone();

        // Take stdout/stderr but don't consume them - let them inherit
        if let Some(stdout) = child.stdout.take() {
            thread::spawn(move || {
                let reader = BufReader::new(stdout);
                for line in reader.lines().map_while(Result::ok) {
                    println!("[Frontend {}] {}", idx + 1, line);
                    std::io::stdout().flush().ok();
                }
                let _ = tx_clone.send(format!("Frontend {} ({}) exited", idx + 1, path));
            });
        }

        if let Some(stderr) = child.stderr.take() {
            thread::spawn(move || {
                let reader = BufReader::new(stderr);
                for line in reader.lines().map_while(Result::ok) {
                    eprintln!("[Frontend {}] {}", idx + 1, line);
                    std::io::stderr().flush().ok();
                }
            });
        }
    }

    let tx_be = tx.clone();
    if let Some(stdout) = backend.stdout.take() {
        thread::spawn(move || {
            let reader = BufReader::new(stdout);
            for line in reader.lines().map_while(Result::ok) {
                println!("[Backend] {}", line);
                std::io::stdout().flush().ok();
            }
            let _ = tx_be.send("Backend exited".to_string());
        });
    }

    if let Some(stderr) = backend.stderr.take() {
        thread::spawn(move || {
            let reader = BufReader::new(stderr);
            for line in reader.lines().map_while(Result::ok) {
                eprintln!("[Backend] {}", line);
                std::io::stderr().flush().ok();
            }
        });
    }

    // Wait for Ctrl+C or process exit
    loop {
        if let Ok(msg) = rx.recv_timeout(Duration::from_millis(100)) {
            eprintln!("\n⚠️  {}", msg);
            break;
        }
    }

    // Cleanup
    for (_, mut child, _) in frontends {
        let _ = child.kill();
    }
    let _ = backend.kill();

    Ok(())
}

fn run_tui(mut frontends: Vec<(usize, Child, String)>, mut backend: Child) -> Result<()> {
    let (tx, rx) = mpsc::channel();

    // Spawn log readers for all frontends
    for (idx, child, _) in frontends.iter_mut() {
        let idx = *idx;
        let tx_fe = tx.clone();
        if let Some(stdout) = child.stdout.take() {
            thread::spawn(move || {
                let reader = BufReader::new(stdout);
                for line in reader.lines().map_while(Result::ok) {
                    let _ = tx_fe.send(LogSource::Frontend(idx, line));
                }
            });
        }

        let tx_fe_err = tx.clone();
        if let Some(stderr) = child.stderr.take() {
            thread::spawn(move || {
                let reader = BufReader::new(stderr);
                for line in reader.lines().map_while(Result::ok) {
                    let _ = tx_fe_err.send(LogSource::Frontend(idx, line));
                }
            });
        }
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

    // Spawn log readers for backend stderr
    let tx_be_err = tx.clone();
    if let Some(stderr) = backend.stderr.take() {
        thread::spawn(move || {
            let reader = BufReader::new(stderr);
            for line in reader.lines().map_while(Result::ok) {
                let _ = tx_be_err.send(LogSource::Backend(line));
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

    let num_frontends = frontends.len();
    let mut frontend_logs: Vec<Vec<String>> = (0..num_frontends).map(|_| Vec::new()).collect();
    let mut backend_logs = Vec::new();
    let mut frontend_exited: Vec<bool> = (0..num_frontends).map(|_| false).collect();
    let mut backend_exited = false;
    let mut scroll_offset = 0usize;

    loop {
        // Collect logs
        while let Ok(log) = rx.try_recv() {
            match log {
                LogSource::Frontend(idx, line) => {
                    if idx < frontend_logs.len() {
                        frontend_logs[idx].push(line);
                    }
                }
                LogSource::Backend(line) => backend_logs.push(line),
            }
        }

        // Draw UI
        terminal.draw(|f| {
            // Create constraints: N frontends + 1 backend + 1 help line
            let mut constraints = vec![];
            let pane_height = 100 / (num_frontends + 1);
            for _ in 0..num_frontends {
                constraints.push(Constraint::Percentage(pane_height as u16));
            }
            constraints.push(Constraint::Percentage(pane_height as u16));
            constraints.push(Constraint::Length(1));

            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints(constraints)
                .split(f.size());

            // Render frontend panes
            for (idx, logs) in frontend_logs.iter().enumerate() {
                let fe_text: Vec<Line> = logs
                    .iter()
                    .rev()
                    .take(chunks[idx].height as usize - 2)
                    .rev()
                    .map(|l| Line::from(Span::raw(l.clone())))
                    .collect();

                let title = format!("Frontend {} ({})", idx + 1, frontends[idx].2);
                let fe_block = Block::default()
                    .title(title)
                    .borders(Borders::ALL)
                    .style(Style::default().fg(Color::Cyan));

                f.render_widget(Paragraph::new(fe_text).block(fe_block), chunks[idx]);
            }

            // Render backend pane with scrolling
            let visible_height = chunks[num_frontends].height as usize - 2;
            let total_lines = backend_logs.len();
            let start_idx = if total_lines > visible_height {
                total_lines.saturating_sub(visible_height + scroll_offset)
            } else {
                0
            };
            let end_idx = total_lines.saturating_sub(scroll_offset);

            let be_text: Vec<Line> = backend_logs[start_idx..end_idx]
                .iter()
                .map(|l| Line::from(Span::raw(l.clone())))
                .collect();

            let scroll_indicator = if scroll_offset > 0 {
                format!(" (↑{} lines)", scroll_offset)
            } else {
                String::new()
            };

            let be_block = Block::default()
                .title(format!("Backend (cargo){}", scroll_indicator))
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::Green));

            f.render_widget(
                Paragraph::new(be_text).block(be_block),
                chunks[num_frontends],
            );

            // Render help line
            let any_exited = frontend_exited.iter().any(|&e| e) || backend_exited;
            let help_text = if any_exited {
                "⚠️  Process exited - Press 'q' to exit | ↑/↓ to scroll"
            } else {
                "Press 'q' to exit | ↑/↓ or PgUp/PgDn to scroll backend logs"
            };
            let help = Paragraph::new(help_text).style(Style::default().fg(if any_exited {
                Color::Yellow
            } else {
                Color::DarkGray
            }));
            f.render_widget(help, chunks[num_frontends + 1]);
        })?;

        // Handle input
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Char('c') | KeyCode::Esc => break,
                    KeyCode::Up => {
                        scroll_offset = scroll_offset.saturating_add(1).min(backend_logs.len());
                    }
                    KeyCode::Down => {
                        scroll_offset = scroll_offset.saturating_sub(1);
                    }
                    KeyCode::PageUp => {
                        scroll_offset = scroll_offset.saturating_add(10).min(backend_logs.len());
                    }
                    KeyCode::PageDown => {
                        scroll_offset = scroll_offset.saturating_sub(10);
                    }
                    KeyCode::Home => {
                        scroll_offset = backend_logs.len();
                    }
                    KeyCode::End => {
                        scroll_offset = 0;
                    }
                    _ => {}
                }
            }
        }

        // Check if processes exited
        for (idx, (_, child, _)) in frontends.iter_mut().enumerate() {
            if !frontend_exited[idx] {
                if let Some(status) = child.try_wait()? {
                    frontend_exited[idx] = true;
                    frontend_logs[idx].push(format!(
                        "⚠️  Frontend process exited with status: {}",
                        status
                    ));
                }
            }
        }
        if !backend_exited {
            if let Some(status) = backend.try_wait()? {
                backend_exited = true;
                backend_logs.push(format!(
                    "⚠️  Backend process exited with status: {}",
                    status
                ));
            }
        }
    }

    // Cleanup
    for (_, mut child, _) in frontends {
        let _ = child.kill();
    }
    let _ = backend.kill();

    crossterm::terminal::disable_raw_mode()?;
    crossterm::execute!(
        terminal.backend_mut(),
        crossterm::terminal::LeaveAlternateScreen,
        crossterm::event::DisableMouseCapture
    )?;

    Ok(())
}

fn infer_spa_config(base_dir: &Path) -> Result<SpaConfig> {
    let working_dir = infer::find_frontend_dir(base_dir)?;
    let abs_working_dir = base_dir.join(&working_dir);
    let output_dir = infer::find_output_dir(&abs_working_dir);

    // Use library's smart inference for dev command and port
    let inferred = heisenberg::utils::infer_from_build_dir(&output_dir)
        .unwrap_or_else(|_| heisenberg::utils::InferredConfig::default_for_dir(&output_dir));

    Ok(SpaConfig {
        name: None,
        working_dir,
        output_dir: output_dir
            .strip_prefix(base_dir)
            .unwrap_or(&output_dir)
            .to_path_buf(),
        dev_command: Some(inferred.dev_command.join(" ")),
        build_command: None,
        dev_server: Some(inferred.dev_url),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;
    use tempfile::TempDir;

    #[test]
    fn test_run_args_parse_extracts_no_tui() {
        let args = RunArgs::parse(vec!["--no-tui".to_string(), "--release".to_string()]);
        assert!(args.no_tui);
        assert_eq!(args.cargo_args, vec!["--release"]);
    }

    #[test]
    fn test_run_args_parse_without_no_tui() {
        let args = RunArgs::parse(vec!["--release".to_string()]);
        assert!(!args.no_tui);
        assert_eq!(args.cargo_args, vec!["--release"]);
    }

    #[test]
    fn test_check_needs_install_no_node_modules() {
        let temp = TempDir::new().unwrap();
        fs::write(temp.path().join("package.json"), "{}").unwrap();

        assert!(check_needs_install(
            &temp.path().join("node_modules"),
            &temp.path().join("package.json")
        ));
    }

    #[test]
    fn test_check_needs_install_stale_node_modules() {
        let temp = TempDir::new().unwrap();
        fs::create_dir_all(temp.path().join("node_modules")).unwrap();
        std::thread::sleep(std::time::Duration::from_millis(10));
        fs::write(temp.path().join("package.json"), "{}").unwrap();

        assert!(check_needs_install(
            &temp.path().join("node_modules"),
            &temp.path().join("package.json")
        ));
    }

    #[test]
    fn test_check_needs_install_fresh_node_modules() {
        let temp = TempDir::new().unwrap();
        fs::write(temp.path().join("package.json"), "{}").unwrap();
        std::thread::sleep(std::time::Duration::from_millis(10));
        fs::create_dir_all(temp.path().join("node_modules")).unwrap();

        assert!(!check_needs_install(
            &temp.path().join("node_modules"),
            &temp.path().join("package.json")
        ));
    }

    #[test]
    fn test_infer_spa_config_sets_working_and_output_dir() {
        let temp = TempDir::new().unwrap();
        fs::create_dir_all(temp.path().join("web")).unwrap();
        fs::write(temp.path().join("web/package.json"), "{}").unwrap();

        let config = infer_spa_config(temp.path()).unwrap();
        assert_eq!(config.working_dir, PathBuf::from("./web"));
    }

    #[test]
    fn test_infer_spa_config_includes_dev_command() {
        let temp = TempDir::new().unwrap();
        fs::create_dir_all(temp.path().join("web")).unwrap();
        fs::write(temp.path().join("web/package.json"), "{}").unwrap();

        let config = infer_spa_config(temp.path()).unwrap();
        assert!(config.dev_command.is_some());
    }

    #[test]
    fn test_infer_spa_config_includes_dev_server() {
        let temp = TempDir::new().unwrap();
        fs::create_dir_all(temp.path().join("web")).unwrap();
        fs::write(temp.path().join("web/package.json"), "{}").unwrap();

        let config = infer_spa_config(temp.path()).unwrap();
        assert!(config.dev_server.is_some());
    }

    #[test]
    fn test_infer_spa_config_no_build_command() {
        let temp = TempDir::new().unwrap();
        fs::create_dir_all(temp.path().join("web")).unwrap();
        fs::write(temp.path().join("web/package.json"), "{}").unwrap();

        let config = infer_spa_config(temp.path()).unwrap();
        assert!(config.build_command.is_none());
    }
}
