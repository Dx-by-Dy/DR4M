

use std::io;

use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Terminal,
    layout::{Constraint, Direction, Layout},
    prelude::CrosstermBackend,
    style::Style,
    text::Line,
    widgets::{Block, Borders, Paragraph, Wrap},
};

#[tokio::main]
async fn main() {
    enable_raw_mode().unwrap();
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen).unwrap();

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).unwrap();

    //let app_backend = app_backend::AppBackend::new();

    let mut app_messages = Vec::new();
    app_messages.push("Welcome to DR4M!".to_string());
    app_messages.push("This is a simple chat application.".to_string());
    app_messages.push("Type your message and press Enter to send.".to_string());

    let mut app_input = String::new();

    loop {
        terminal
            .draw(|frame| {
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Min(1), Constraint::Length(3)])
                    .split(frame.area());

                let text: Vec<Line> = app_messages.iter().map(|m| Line::from(m.clone())).collect();

                let messages = Paragraph::new(text).wrap(Wrap { trim: true }).block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(Style::new().dark_gray())
                        .title("Output"),
                );

                frame.render_widget(messages, chunks[0]);

                let input = Paragraph::new(app_input.as_str())
                    .wrap(Wrap { trim: true })
                    .block(Block::default().borders(Borders::ALL).title("Input"));

                frame.render_widget(input, chunks[1]);

                frame.set_cursor_position((
                    chunks[1].x + app_input.len() as u16 + 1,
                    chunks[1].y + 1,
                ));
            })
            .unwrap();

        if event::poll(std::time::Duration::from_millis(100)).unwrap() {
            if let Event::Key(key) = event::read().unwrap() {
                match key.code {
                    KeyCode::Char(c) => app_input.push(c),
                    KeyCode::Backspace => {
                        app_input.pop();
                    }
                    KeyCode::Enter => {} // app.on_enter(),
                    KeyCode::Esc => break,
                    _ => {}
                }
            }
        }
    }

    disable_raw_mode().unwrap();
    execute!(terminal.backend_mut(), LeaveAlternateScreen).unwrap();
    terminal.show_cursor().unwrap();
}
