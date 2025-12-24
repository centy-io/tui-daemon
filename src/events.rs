use std::time::Duration;

use crossterm::event::{self, Event as CrosstermEvent, KeyEvent, MouseEvent};
use tokio::sync::mpsc;

/// Application events
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum Event {
    /// Terminal tick for UI refresh
    Tick,
    /// Keyboard input
    Key(KeyEvent),
    /// Mouse input (reserved for future mouse support)
    Mouse(MouseEvent),
    /// Terminal resize (handled automatically by ratatui)
    Resize(u16, u16),
}

/// Event handler that polls for terminal events
pub struct EventHandler {
    /// Event receiver
    rx: mpsc::UnboundedReceiver<Event>,
    /// Event sender (kept for potential future use)
    _tx: mpsc::UnboundedSender<Event>,
}

impl EventHandler {
    /// Create a new event handler with the specified tick rate
    pub fn new(tick_rate: Duration) -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        let event_tx = tx.clone();

        // Spawn event polling task
        tokio::spawn(async move {
            let mut last_tick = std::time::Instant::now();
            loop {
                // Calculate timeout for next tick
                let timeout = tick_rate
                    .checked_sub(last_tick.elapsed())
                    .unwrap_or(Duration::ZERO);

                // Poll for events
                if event::poll(timeout).unwrap_or(false) {
                    match event::read() {
                        Ok(CrosstermEvent::Key(key)) => {
                            if event_tx.send(Event::Key(key)).is_err() {
                                break;
                            }
                        }
                        Ok(CrosstermEvent::Mouse(mouse)) => {
                            if event_tx.send(Event::Mouse(mouse)).is_err() {
                                break;
                            }
                        }
                        Ok(CrosstermEvent::Resize(w, h)) => {
                            if event_tx.send(Event::Resize(w, h)).is_err() {
                                break;
                            }
                        }
                        _ => {}
                    }
                }

                // Send tick event
                if last_tick.elapsed() >= tick_rate {
                    if event_tx.send(Event::Tick).is_err() {
                        break;
                    }
                    last_tick = std::time::Instant::now();
                }
            }
        });

        Self { rx, _tx: tx }
    }

    /// Receive the next event
    pub async fn next(&mut self) -> Option<Event> {
        self.rx.recv().await
    }
}
