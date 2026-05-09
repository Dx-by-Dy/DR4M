pub mod render_callback;
pub mod render_state;

use crate::{
    connection::{Connected, Connection, ConnectionBehaviour},
    controller::{
        component::{Component, Quit},
        control_event::ControlEventHook,
    },
    inputter::input_event::InputEventBehaviour,
    ui::{render_callback::RenderBehaviour, render_state::RenderState},
};
use crossterm::{
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use logger::{async_log, log_entry::LogEntry};
use ratatui::{Terminal, backend::CrosstermBackend};
use tokio::select;
use tokio_util::sync::CancellationToken;

pub struct UI {
    connection: Connection,
    terminal: Terminal<CrosstermBackend<std::io::Stdout>>,
    cancellation_token: CancellationToken,
}

impl RenderBehaviour for UI {
    fn handle_render_state(&mut self, render_state: RenderState) -> impl Future<Output = ()> {
        async {
            if let Err(e) = self.terminal.draw(|frame| render_state.render_frame(frame)) {
                async_log!(LogEntry::from(
                    format!("UIManager update error: {:?}", e).as_bytes()
                ));
            }
        }
    }
}

impl Connected for UI {
    fn connection(&mut self) -> &mut Connection {
        &mut self.connection
    }
}

impl InputEventBehaviour for UI {}

impl ControlEventHook for UI {
    fn quit_hook(&mut self) -> impl Future<Output = ()> {
        async {
            _ = disable_raw_mode();
            _ = execute!(self.terminal.backend_mut(), LeaveAlternateScreen);
        }
    }
}

impl Quit for UI {
    fn cancellation_token(&self) -> &CancellationToken {
        &self.cancellation_token
    }
}

impl ConnectionBehaviour for UI {}

impl Component for UI {
    fn new(connection: Connection) -> Self {
        enable_raw_mode().unwrap();
        let mut stdout = std::io::stdout();
        execute!(stdout, EnterAlternateScreen).unwrap();

        Self {
            terminal: Terminal::new(CrosstermBackend::new(stdout)).unwrap(),
            connection,
            cancellation_token: CancellationToken::new(),
        }
    }

    fn main_loop(mut self) -> impl Future<Output = ()> + Send + 'static {
        async move {
            loop {
                select! {
                    connection_event = self.connection.recv() => {
                        self.release(connection_event).await;
                    }
                    _ = self.cancellation_token.cancelled() => {
                        break;
                    }
                }
            }
        }
    }
}
