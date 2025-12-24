use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, List, ListItem, Paragraph, Wrap},
    Frame,
};

use crate::app::{App, ConnectionStatus, ControlAction, FocusedPanel};

/// Render the main dashboard
pub fn render_dashboard(frame: &mut Frame, app: &App) {
    // Create the main layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Header
            Constraint::Min(10),    // Main content
            Constraint::Length(3),  // Footer
        ])
        .split(frame.area());

    render_header(frame, app, chunks[0]);
    render_main_content(frame, app, chunks[1]);
    render_footer(frame, app, chunks[2]);
}

/// Render the header with title and connection status
fn render_header(frame: &mut Frame, app: &App, area: Rect) {
    let (status_text, status_color) = match &app.connection_status {
        ConnectionStatus::Connected => ("Connected", Color::Green),
        ConnectionStatus::Connecting => ("Connecting...", Color::Yellow),
        ConnectionStatus::Disconnected => ("Disconnected", Color::Red),
        ConnectionStatus::Error(msg) => (msg.as_str(), Color::Red),
    };

    let header = Paragraph::new(Line::from(vec![
        Span::styled(
            " Daemon Controller ",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" | "),
        Span::styled(format!(" {} ", status_text), Style::default().fg(status_color)),
        Span::raw(" | "),
        Span::raw(format!(" {} ", app.daemon_address)),
    ]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan)),
    );

    frame.render_widget(header, area);
}

/// Render the main content area with panels
fn render_main_content(frame: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(40), // Left: Status + Metrics
            Constraint::Percentage(30), // Center: Controls
            Constraint::Percentage(30), // Right: Logs
        ])
        .split(area);

    // Left column: Status and Metrics
    let left_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[0]);

    render_status_panel(frame, app, left_chunks[0]);
    render_metrics_panel(frame, app, left_chunks[1]);

    // Center: Controls
    render_controls_panel(frame, app, chunks[1]);

    // Right: Logs
    render_logs_panel(frame, app, chunks[2]);
}

/// Render the daemon status panel
fn render_status_panel(frame: &mut Frame, app: &App, area: Rect) {
    let is_focused = app.focused_panel == FocusedPanel::Status;
    let border_style = if is_focused {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::White)
    };

    let status_info = if let Some(status) = &app.daemon_status {
        vec![
            Line::from(vec![
                Span::raw("State: "),
                Span::styled(
                    app.daemon_state_string(),
                    Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(format!("Version: {}", status.version)),
            Line::from(format!("Uptime: {}s", status.uptime_seconds)),
            Line::from(format!("Message: {}", status.message)),
        ]
    } else {
        vec![Line::from(Span::styled(
            "No data available",
            Style::default().fg(Color::DarkGray),
        ))]
    };

    let status_block = Paragraph::new(status_info)
        .block(
            Block::default()
                .title(" Status ")
                .borders(Borders::ALL)
                .border_style(border_style),
        )
        .wrap(Wrap { trim: true });

    frame.render_widget(status_block, area);
}

/// Render the metrics panel with gauges
fn render_metrics_panel(frame: &mut Frame, app: &App, area: Rect) {
    let inner_area = {
        let block = Block::default()
            .title(" Metrics ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::White));
        let inner = block.inner(area);
        frame.render_widget(block, area);
        inner
    };

    if let Some(metrics) = &app.daemon_metrics {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(2),
                Constraint::Length(2),
                Constraint::Min(1),
            ])
            .margin(1)
            .split(inner_area);

        // CPU gauge
        let cpu_pct = metrics.cpu_usage_percent.clamp(0.0, 100.0);
        let cpu_gauge = Gauge::default()
            .label(format!("CPU: {:.1}%", cpu_pct))
            .gauge_style(Style::default().fg(Color::Cyan))
            .ratio(cpu_pct / 100.0);
        frame.render_widget(cpu_gauge, chunks[0]);

        // Memory gauge
        let mem_pct = if metrics.memory_limit_bytes > 0 {
            (metrics.memory_bytes as f64 / metrics.memory_limit_bytes as f64 * 100.0).clamp(0.0, 100.0)
        } else {
            0.0
        };
        let memory_gauge = Gauge::default()
            .label(format!(
                "Memory: {} / {} ({:.1}%)",
                format_bytes(metrics.memory_bytes),
                format_bytes(metrics.memory_limit_bytes),
                mem_pct
            ))
            .gauge_style(Style::default().fg(Color::Magenta))
            .ratio(mem_pct / 100.0);
        frame.render_widget(memory_gauge, chunks[1]);

        // Stats
        let stats = Paragraph::new(vec![
            Line::from(format!("Connections: {}", metrics.connections_active)),
            Line::from(format!("Requests: {}", metrics.requests_total)),
            Line::from(format!("Errors: {}", metrics.errors_total)),
        ]);
        frame.render_widget(stats, chunks[2]);
    } else {
        let no_data = Paragraph::new("No metrics available")
            .style(Style::default().fg(Color::DarkGray));
        frame.render_widget(no_data, inner_area);
    }
}

/// Render the controls panel
fn render_controls_panel(frame: &mut Frame, app: &App, area: Rect) {
    let is_focused = app.focused_panel == FocusedPanel::Controls;
    let border_style = if is_focused {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::White)
    };

    let items: Vec<ListItem> = ControlAction::ALL
        .iter()
        .enumerate()
        .map(|(i, action)| {
            let style = if i == app.selected_action && is_focused {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else if i == app.selected_action {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            ListItem::new(format!("  {}  ", action.label())).style(style)
        })
        .collect();

    let controls_list = List::new(items).block(
        Block::default()
            .title(" Controls ")
            .borders(Borders::ALL)
            .border_style(border_style),
    );

    frame.render_widget(controls_list, area);
}

/// Render the logs panel
fn render_logs_panel(frame: &mut Frame, app: &App, area: Rect) {
    let is_focused = app.focused_panel == FocusedPanel::Logs;
    let border_style = if is_focused {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::White)
    };

    let items: Vec<ListItem> = app
        .logs
        .iter()
        .skip(app.log_scroll)
        .take(area.height.saturating_sub(2) as usize)
        .map(|log| {
            let level_color = match log.level.as_str() {
                "ERROR" => Color::Red,
                "WARN" => Color::Yellow,
                "INFO" => Color::Green,
                _ => Color::Gray,
            };
            ListItem::new(Line::from(vec![
                Span::styled(
                    format!("[{}] ", log.timestamp),
                    Style::default().fg(Color::DarkGray),
                ),
                Span::styled(
                    format!("{:<5} ", log.level),
                    Style::default().fg(level_color),
                ),
                Span::raw(&log.message),
            ]))
        })
        .collect();

    let logs_list = List::new(items).block(
        Block::default()
            .title(format!(" Logs ({}) ", app.logs.len()))
            .borders(Borders::ALL)
            .border_style(border_style),
    );

    frame.render_widget(logs_list, area);
}

/// Render the footer with keybindings
fn render_footer(frame: &mut Frame, app: &App, area: Rect) {
    let keybindings = if let Some(msg) = &app.status_message {
        Line::from(Span::styled(
            msg.clone(),
            Style::default().fg(Color::Yellow),
        ))
    } else {
        Line::from(vec![
            Span::styled(" q ", Style::default().fg(Color::Red)),
            Span::raw("Quit"),
            Span::raw(" | "),
            Span::styled(" Tab ", Style::default().fg(Color::Cyan)),
            Span::raw("Switch Panel"),
            Span::raw(" | "),
            Span::styled(" c ", Style::default().fg(Color::Green)),
            Span::raw("Connect"),
            Span::raw(" | "),
            Span::styled(" Enter ", Style::default().fg(Color::Yellow)),
            Span::raw("Execute"),
            Span::raw(" | "),
            Span::styled(" j/k ", Style::default().fg(Color::Magenta)),
            Span::raw("Navigate"),
        ])
    };

    let footer = Paragraph::new(keybindings)
        .block(Block::default().borders(Borders::ALL));

    frame.render_widget(footer, area);
}

/// Format bytes to human-readable string
fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.1}GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1}MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1}KB", bytes as f64 / KB as f64)
    } else {
        format!("{}B", bytes)
    }
}
