use std::io;

use crossterm::{
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use logger::{async_log_quiet, log_entry::LogEntry};
use ratatui::{Frame, Terminal, backend::CrosstermBackend};

use crate::{
    connection::{Connected, Connection, ConnectionBehaviour},
    controller::control_event::ControlEventHook,
    inputter::input_event::InputEventBehaviour,
    ui::render_callback::RenderCallbackBehaviour,
};

pub struct UI {
    connection: Connection,
    terminal: Terminal<CrosstermBackend<std::io::Stdout>>,
}

impl UI {
    pub fn new(connection: Connection) -> io::Result<Self> {
        enable_raw_mode()?;
        let mut stdout = std::io::stdout();
        execute!(stdout, EnterAlternateScreen)?;

        Ok(Self {
            terminal: Terminal::new(CrosstermBackend::new(stdout))?,
            connection,
        })
    }

    pub async fn spawn_task(self) -> tokio::task::JoinHandle<()> {
        tokio::spawn(self.start())
    }

    async fn start(mut self) {
        async_log_quiet!(LogEntry::from("UIManager started".as_bytes())).await;
        loop {
            let connection_event = self.connection.recv().await;
            self.release(connection_event).await;
        }
    }

    // async fn update(&mut self, render_callback: RenderCallback) {
    //     match render_callback {
    //         // RenderCallback::High(paragraph) => {
    //         //     if let Err(e) = self.terminal.draw(|frame: &mut Frame<'_>| {
    //         //         let chunks = Layout::default()
    //         //             .direction(Direction::Vertical)
    //         //             .constraints([Constraint::Min(1), Constraint::Length(3)])
    //         //             .split(frame.area());
    //         //         frame.render_widget(paragraph, chunks[0]);
    //         //     }) {
    //         //         async_log_quiet!(LogEntry::from(
    //         //             format!("UIManager update error: {:?}", e).as_bytes()
    //         //         ))
    //         //         .await;
    //         //     }
    //         // }
    //         // RenderCallback::Bottom((paragraph, position)) => {
    //         //     if let Err(e) = self.terminal.draw(|frame| {
    //         //         let chunks = Layout::default()
    //         //             .direction(Direction::Vertical)
    //         //             .constraints([Constraint::Min(1), Constraint::Length(3)])
    //         //             .split(frame.area());
    //         //         frame.render_widget(paragraph, chunks[1]);
    //         //         frame.set_cursor_position(position);
    //         //     }) {
    //         //         async_log_quiet!(LogEntry::from(
    //         //             format!("UIManager update error: {:?}", e).as_bytes()
    //         //         ))
    //         //         .await;
    //         //     }
    //         // }
    //         RenderCallback::Redraw => unimplemented!(),
    //         RenderCallback::Render(render) => {
    //             if let Err(e) = self.terminal.draw(render) {
    //                 async_log_quiet!(LogEntry::from(
    //                     format!("UIManager update error: {:?}", e).as_bytes()
    //                 ))
    //                 .await;
    //             }
    //         }
    //     }
    // }
}

impl RenderCallbackBehaviour for UI {
    fn render(
        &mut self,
        render_callback: Box<dyn FnOnce(&mut Frame<'_>) + Send + 'static>,
    ) -> impl Future<Output = ()> {
        async {
            if let Err(e) = self.terminal.draw(render_callback) {
                async_log_quiet!(LogEntry::from(
                    format!("UIManager update error: {:?}", e).as_bytes()
                ))
                .await;
            }
        }
    }

    fn redraw(&mut self) -> impl Future<Output = ()> {
        async { todo!() }
    }
}

impl Connected for UI {
    fn connection(&mut self) -> &mut Connection {
        &mut self.connection
    }
}

impl InputEventBehaviour for UI {}

impl ControlEventHook for UI {}

impl ConnectionBehaviour for UI {}

impl Drop for UI {
    fn drop(&mut self) {
        _ = disable_raw_mode();
        _ = execute!(self.terminal.backend_mut(), LeaveAlternateScreen);
    }
}
