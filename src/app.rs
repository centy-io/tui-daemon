use std::time::Instant;

use crate::grpc::daemon::{ControlCommand, DaemonState, MetricsResponse, StatusResponse};

/// Represents the connection status to the daemon
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum ConnectionStatus {
    #[default]
    Disconnected,
    Connecting,
    Connected,
    Error(String),
}

/// The currently focused panel in the UI
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FocusedPanel {
    #[default]
    Status,
    Controls,
    Logs,
}

impl FocusedPanel {
    pub fn next(self) -> Self {
        match self {
            Self::Status => Self::Controls,
            Self::Controls => Self::Logs,
            Self::Logs => Self::Status,
        }
    }

    pub fn prev(self) -> Self {
        match self {
            Self::Status => Self::Logs,
            Self::Controls => Self::Status,
            Self::Logs => Self::Controls,
        }
    }
}

/// Available control actions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControlAction {
    Start,
    Stop,
    Restart,
    Reload,
}

impl ControlAction {
    pub const ALL: [ControlAction; 4] = [
        ControlAction::Start,
        ControlAction::Stop,
        ControlAction::Restart,
        ControlAction::Reload,
    ];

    pub fn label(self) -> &'static str {
        match self {
            ControlAction::Start => "Start",
            ControlAction::Stop => "Stop",
            ControlAction::Restart => "Restart",
            ControlAction::Reload => "Reload",
        }
    }

    pub fn to_command(self) -> ControlCommand {
        match self {
            ControlAction::Start => ControlCommand::Start,
            ControlAction::Stop => ControlCommand::Stop,
            ControlAction::Restart => ControlCommand::Restart,
            ControlAction::Reload => ControlCommand::Reload,
        }
    }
}

/// A log entry for display
#[derive(Debug, Clone)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: String,
    pub message: String,
}

/// Main application state
#[derive(Debug)]
pub struct App {
    /// Whether the application should quit
    pub should_quit: bool,

    /// Connection status to the daemon
    pub connection_status: ConnectionStatus,

    /// Currently focused panel
    pub focused_panel: FocusedPanel,

    /// Selected control action index
    pub selected_action: usize,

    /// Latest daemon status
    pub daemon_status: Option<StatusResponse>,

    /// Latest daemon metrics
    pub daemon_metrics: Option<MetricsResponse>,

    /// Log entries
    pub logs: Vec<LogEntry>,

    /// Log scroll offset
    pub log_scroll: usize,

    /// gRPC endpoint address
    pub daemon_address: String,

    /// When the app started (reserved for future uptime display)
    #[allow(dead_code)]
    pub start_time: Instant,

    /// Last status message
    pub status_message: Option<String>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            should_quit: false,
            connection_status: ConnectionStatus::default(),
            focused_panel: FocusedPanel::default(),
            selected_action: 0,
            daemon_status: None,
            daemon_metrics: None,
            logs: Vec::new(),
            log_scroll: 0,
            daemon_address: "http://[::1]:50051".to_string(),
            start_time: Instant::now(),
            status_message: None,
        }
    }
}

impl App {
    /// Create a new App with the specified daemon address
    pub fn new(daemon_address: String) -> Self {
        Self {
            daemon_address,
            ..Default::default()
        }
    }

    /// Request to quit the application
    pub fn quit(&mut self) {
        self.should_quit = true;
    }

    /// Move focus to the next panel
    pub fn focus_next(&mut self) {
        self.focused_panel = self.focused_panel.next();
    }

    /// Move focus to the previous panel
    pub fn focus_prev(&mut self) {
        self.focused_panel = self.focused_panel.prev();
    }

    /// Select the next control action
    pub fn select_next_action(&mut self) {
        if self.selected_action < ControlAction::ALL.len() - 1 {
            self.selected_action += 1;
        }
    }

    /// Select the previous control action
    pub fn select_prev_action(&mut self) {
        if self.selected_action > 0 {
            self.selected_action -= 1;
        }
    }

    /// Get the currently selected action
    pub fn current_action(&self) -> ControlAction {
        ControlAction::ALL[self.selected_action]
    }

    /// Scroll logs up
    pub fn scroll_logs_up(&mut self) {
        if self.log_scroll > 0 {
            self.log_scroll -= 1;
        }
    }

    /// Scroll logs down
    pub fn scroll_logs_down(&mut self) {
        if self.log_scroll < self.logs.len().saturating_sub(1) {
            self.log_scroll += 1;
        }
    }

    /// Add a log entry
    pub fn add_log(&mut self, level: &str, message: String) {
        let timestamp = chrono::Local::now().format("%H:%M:%S").to_string();
        self.logs.push(LogEntry {
            timestamp,
            level: level.to_string(),
            message,
        });
        // Auto-scroll to bottom
        self.log_scroll = self.logs.len().saturating_sub(1);
    }

    /// Update connection status
    pub fn set_connection_status(&mut self, status: ConnectionStatus) {
        self.connection_status = status;
    }

    /// Update daemon status
    pub fn update_status(&mut self, status: StatusResponse) {
        self.daemon_status = Some(status);
    }

    /// Update daemon metrics
    pub fn update_metrics(&mut self, metrics: MetricsResponse) {
        self.daemon_metrics = Some(metrics);
    }

    /// Set a status message to display
    #[allow(dead_code)]
    pub fn set_status_message(&mut self, message: String) {
        self.status_message = Some(message);
    }

    /// Clear the status message
    #[allow(dead_code)]
    pub fn clear_status_message(&mut self) {
        self.status_message = None;
    }

    /// Get the daemon state as a string
    pub fn daemon_state_string(&self) -> &str {
        match &self.daemon_status {
            Some(status) => match DaemonState::try_from(status.state) {
                Ok(DaemonState::Unknown) => "Unknown",
                Ok(DaemonState::Starting) => "Starting",
                Ok(DaemonState::Running) => "Running",
                Ok(DaemonState::Stopping) => "Stopping",
                Ok(DaemonState::Stopped) => "Stopped",
                Ok(DaemonState::Error) => "Error",
                Err(_) => "Invalid",
            },
            None => "N/A",
        }
    }
}
