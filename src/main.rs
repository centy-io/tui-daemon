mod app;
mod events;
mod grpc;
mod ui;

use std::time::Duration;

use app::{App, ConnectionStatus};
use color_eyre::Result;
use crossterm::event::{KeyCode, KeyModifiers};
use events::{Event, EventHandler};
use grpc::DaemonClient;
use ratatui::DefaultTerminal;
use ui::render_dashboard;

/// Tick rate for UI refresh (in milliseconds)
const TICK_RATE_MS: u64 = 250;

/// Default daemon address
const DEFAULT_DAEMON_ADDRESS: &str = "http://127.0.0.1:50051";

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize error handling
    color_eyre::install()?;

    // Parse command-line args (simple version)
    let daemon_address = std::env::args()
        .nth(1)
        .unwrap_or_else(|| DEFAULT_DAEMON_ADDRESS.to_string());

    // Raw mode + alternate screen, plus a panic hook that restores the terminal.
    // Must run after color_eyre::install() so the eyre hook is wrapped, not replaced.
    let mut terminal = ratatui::init();

    // Create app and run
    let mut app = App::new(daemon_address.clone());
    let mut client = DaemonClient::new(daemon_address);

    app.add_log("INFO", "Daemon Controller started".to_string());
    app.add_log("INFO", format!("Target: {}", app.daemon_address));

    let result = run_app(&mut terminal, &mut app, &mut client).await;

    ratatui::restore();

    result
}

/// Main application loop
async fn run_app(
    terminal: &mut DefaultTerminal,
    app: &mut App,
    client: &mut DaemonClient,
) -> Result<()> {
    let mut events = EventHandler::new(Duration::from_millis(TICK_RATE_MS));

    loop {
        // Draw UI
        terminal.draw(|frame| render_dashboard(frame, app))?;

        // Handle events
        if let Some(event) = events.next().await {
            match event {
                Event::Key(key) => {
                    handle_key_event(app, client, key.code, key.modifiers).await;
                }
                Event::Tick => {
                    // Periodic update - refresh data if connected
                    if client.is_connected() {
                        refresh_data(app, client).await;
                    }
                }
                Event::Resize(_, _) => {
                    // Terminal will re-render automatically
                }
                Event::Mouse(_) => {
                    // Mouse events handled here if needed
                }
            }
        }

        // Check if we should quit
        if app.should_quit {
            break;
        }
    }

    Ok(())
}

/// Handle keyboard input
async fn handle_key_event(
    app: &mut App,
    client: &mut DaemonClient,
    code: KeyCode,
    modifiers: KeyModifiers,
) {
    // Global keybindings
    match code {
        KeyCode::Char('q' | 'Q') => {
            app.quit();
            return;
        }
        KeyCode::Char('c') if modifiers.contains(KeyModifiers::CONTROL) => {
            app.quit();
            return;
        }
        KeyCode::Tab => {
            app.focus_next();
            return;
        }
        KeyCode::BackTab => {
            app.focus_prev();
            return;
        }
        KeyCode::Char('c' | 'C') => {
            connect_to_daemon(app, client).await;
            return;
        }
        KeyCode::Char('d' | 'D') => {
            disconnect_from_daemon(app, client);
            return;
        }
        _ => {}
    }

    // Panel-specific keybindings
    match app.focused_panel {
        app::FocusedPanel::Controls => match code {
            KeyCode::Up | KeyCode::Char('k') => app.select_prev_action(),
            KeyCode::Down | KeyCode::Char('j') => app.select_next_action(),
            KeyCode::Enter => {
                execute_action(app, client).await;
            }
            _ => {}
        },
        app::FocusedPanel::Logs => match code {
            KeyCode::Up | KeyCode::Char('k') => app.scroll_logs_up(),
            KeyCode::Down | KeyCode::Char('j') => app.scroll_logs_down(),
            _ => {}
        },
        app::FocusedPanel::Status => {
            // Status panel has no specific actions
        }
    }
}

/// Connect to the daemon
async fn connect_to_daemon(app: &mut App, client: &mut DaemonClient) {
    if client.is_connected() {
        app.add_log("WARN", "Already connected".to_string());
        return;
    }

    app.set_connection_status(ConnectionStatus::Connecting);
    app.add_log("INFO", "Connecting to daemon...".to_string());

    match client.connect().await {
        Ok(()) => {
            app.set_connection_status(ConnectionStatus::Connected);
            app.add_log("INFO", "Connected successfully".to_string());
            // Fetch initial data
            refresh_data(app, client).await;
        }
        Err(e) => {
            app.set_connection_status(ConnectionStatus::Error("Connection failed".to_string()));
            app.add_log("ERROR", format!("Connection failed: {e}"));
        }
    }
}

/// Disconnect from the daemon
fn disconnect_from_daemon(app: &mut App, client: &mut DaemonClient) {
    if !client.is_connected() {
        app.add_log("WARN", "Not connected".to_string());
        return;
    }

    client.disconnect();
    app.set_connection_status(ConnectionStatus::Disconnected);
    app.daemon_status = None;
    app.daemon_metrics = None;
    app.add_log("INFO", "Disconnected from daemon".to_string());
}

/// Execute the selected control action
async fn execute_action(app: &mut App, client: &mut DaemonClient) {
    if !client.is_connected() {
        app.add_log("WARN", "Not connected - press 'c' to connect".to_string());
        return;
    }

    let action = app.current_action();
    app.add_log("INFO", format!("Executing: {}", action.label()));

    match client.control(action.to_command()).await {
        Ok(response) => {
            if response.success {
                app.add_log("INFO", format!("Success: {}", response.message));
            } else {
                app.add_log("WARN", format!("Failed: {}", response.message));
            }
        }
        Err(e) => {
            app.add_log("ERROR", format!("Command failed: {e}"));
        }
    }
}

/// Refresh status and metrics from daemon
async fn refresh_data(app: &mut App, client: &mut DaemonClient) {
    // Get status
    match client.get_status().await {
        Ok(status) => {
            app.update_status(status);
        }
        Err(e) => {
            app.add_log("ERROR", format!("Failed to get status: {e}"));
        }
    }

    // Get metrics
    match client.get_metrics().await {
        Ok(metrics) => {
            app.update_metrics(metrics);
        }
        Err(e) => {
            app.add_log("ERROR", format!("Failed to get metrics: {e}"));
        }
    }
}
