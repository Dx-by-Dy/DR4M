use std::collections::VecDeque;

use crate::{
    control_manager::{ControlEvent, ControlEventBehaviour},
    ui_manager::RenderCallback,
};
use crossterm::event::KeyEvent;
use logger::{async_log_quiet, async_read_log, log_entry::LogEntry};
use ratatui::{
    style::Style,
    text::Line,
    widgets::{Block, Borders, Paragraph, Wrap},
};
use tokio::{
    select,
    sync::{mpsc, oneshot},
    task::JoinHandle,
};

pub enum ConnectionEvent {
    KeyEvent(KeyEvent),
    TopRenderCallback(RenderCallback),
    BottomRenderCallback(RenderCallback),
    ControlEvent(ControlEvent),
}

pub struct Connection {
    pub top_render_sender: Option<mpsc::Sender<RenderCallback>>,
    pub bottom_render_sender: Option<mpsc::Sender<RenderCallback>>,
    pub top_render_receiver: Option<mpsc::Receiver<RenderCallback>>,
    pub bottom_render_receiver: Option<mpsc::Receiver<RenderCallback>>,
    pub key_event_sender: Option<mpsc::Sender<KeyEvent>>,
    pub key_event_receiver: Option<mpsc::Receiver<KeyEvent>>,
    pub control_receiver: mpsc::Receiver<ControlEvent>,
}

impl Connection {
    pub fn new(
        top_render_sender: Option<mpsc::Sender<RenderCallback>>,
        bottom_render_sender: Option<mpsc::Sender<RenderCallback>>,
        top_render_receiver: Option<mpsc::Receiver<RenderCallback>>,
        bottom_render_receiver: Option<mpsc::Receiver<RenderCallback>>,
        key_event_sender: Option<mpsc::Sender<KeyEvent>>,
        key_event_receiver: Option<mpsc::Receiver<KeyEvent>>,
        control_receiver: mpsc::Receiver<ControlEvent>,
    ) -> Self {
        Self {
            top_render_sender,
            bottom_render_sender,
            top_render_receiver,
            bottom_render_receiver,
            key_event_sender,
            key_event_receiver,
            control_receiver,
        }
    }

    async fn recv(&mut self) -> ConnectionEvent {
        select! {
            Some(event) = async {
                if let Some(receiver) = self.key_event_receiver.as_mut() {
                    receiver.recv().await
                } else {
                    futures::future::pending().await
                }
            } => {
                ConnectionEvent::KeyEvent(event)
            }

            Some(render) = async {
                if let Some(receiver) = self.bottom_render_receiver.as_mut() {
                    receiver.recv().await
                } else {
                    futures::future::pending().await
                }
            } => {
                ConnectionEvent::BottomRenderCallback(render)
            }

            Some(render) = async {
                if let Some(receiver) = self.top_render_receiver.as_mut() {
                    receiver.recv().await
                } else {
                    futures::future::pending().await
                }
            } => {
                ConnectionEvent::TopRenderCallback(render)
            }

            Some(control) = self.control_receiver.recv() => {
                ConnectionEvent::ControlEvent(control)
            }
        }
    }

    pub async fn top_render_try_send(
        &mut self,
        render: RenderCallback,
    ) -> Result<bool, mpsc::error::SendError<RenderCallback>> {
        if let Some(sender) = self.top_render_sender.as_mut() {
            sender.send(render).await.map(|_| true)
        } else {
            Ok(false)
        }
    }

    pub async fn bottom_render_try_send(
        &mut self,
        render: RenderCallback,
    ) -> Result<bool, mpsc::error::SendError<RenderCallback>> {
        if let Some(sender) = self.bottom_render_sender.as_mut() {
            sender.send(render).await.map(|_| true)
        } else {
            Ok(false)
        }
    }

    pub async fn key_event_try_send(
        &mut self,
        event: KeyEvent,
    ) -> Result<bool, mpsc::error::SendError<KeyEvent>> {
        if let Some(sender) = self.key_event_sender.as_mut() {
            sender.send(event).await.map(|_| true)
        } else {
            Ok(false)
        }
    }
}

pub struct LoggerManager {
    connection: Connection,
    buffer: VecDeque<Line<'static>>,
}

impl LoggerManager {
    pub fn new(connection: Connection) -> Self {
        Self {
            connection,
            buffer: VecDeque::with_capacity(10_000),
        }
    }

    pub async fn spawn_task(self) -> JoinHandle<()> {
        tokio::spawn(self.start())
    }

    async fn start(mut self) {
        loop {
            select! {
                connection_event = self.connection.recv() => {
                    self.handle_connection_event(connection_event).await;
                }
                Ok(log_event) = async_read_log!() => {
                    self.handle_log_event(log_event).await;
                }
            }
        }
    }

    async fn handle_connection_event(&mut self, event: ConnectionEvent) {
        match event {
            ConnectionEvent::KeyEvent(key_event) => self.handle_key_event(key_event),
            ConnectionEvent::ControlEvent(control_event) => control_event.release(self).await,
            _ => unreachable!(),
        }
    }

    async fn handle_log_event(&mut self, event: LogEntry) {
        if self.buffer.len() == self.buffer.capacity() {
            self.buffer.pop_back();
        }
        self.buffer.push_front(Line::from(format!("{}", event)));
        self.render().await;
    }

    async fn render(&mut self) {
        let style = match self.connection.top_render_sender.is_some() {
            true => Style::new(),
            false => Style::new().dark_gray(),
        };
        let render = Paragraph::new(self.buffer.iter().cloned().collect::<Vec<Line>>())
            .wrap(Wrap { trim: true })
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(style)
                    .title("Output"),
            );
        if let Err(e) = self
            .connection
            .top_render_try_send(RenderCallback::High(render))
            .await
        {
            async_log_quiet!(LogEntry::from(
                format!("LoggerManager send error: {:?}", e).as_bytes()
            ))
            .await;
        }
    }

    fn handle_key_event(&mut self, _event: KeyEvent) {}
}

impl ControlEventBehaviour for LoggerManager {
    fn send_render_channel(
        &mut self,
        bridge: oneshot::Sender<mpsc::Sender<RenderCallback>>,
    ) -> impl Future<Output = ()> {
        async {
            if let Err(e) = bridge.send(self.connection.top_render_sender.take().unwrap()) {
                async_log_quiet!(LogEntry::from(
                    format!("LoggerManager send error: {:?}", e).as_bytes()
                ))
                .await;
            }
        }
    }

    fn recv_render_channel(
        &mut self,
        bridge: oneshot::Receiver<mpsc::Sender<RenderCallback>>,
    ) -> impl Future<Output = ()> {
        async {
            match bridge.await {
                Ok(channel) => {
                    self.connection.top_render_sender = Some(channel);
                    self.render().await;
                }
                Err(e) => {
                    async_log_quiet!(LogEntry::from(
                        format!("LoggerManager recv error: {:?}", e).as_bytes()
                    ))
                    .await;
                }
            }
        }
    }

    fn send_event_channel(
        &mut self,
        bridge: oneshot::Sender<mpsc::Receiver<KeyEvent>>,
    ) -> impl Future<Output = ()> {
        async {
            if let Err(e) = bridge.send(self.connection.key_event_receiver.take().unwrap()) {
                async_log_quiet!(LogEntry::from(
                    format!("LoggerManager send error: {:?}", e).as_bytes()
                ))
                .await;
            }
        }
    }

    fn recv_event_channel(
        &mut self,
        bridge: oneshot::Receiver<mpsc::Receiver<KeyEvent>>,
    ) -> impl Future<Output = ()> {
        async {
            match bridge.await {
                Ok(channel) => {
                    self.connection.key_event_receiver = Some(channel);
                }
                Err(e) => {
                    async_log_quiet!(LogEntry::from(
                        format!("LoggerManager recv error: {:?}", e).as_bytes()
                    ))
                    .await;
                }
            }
        }
    }
}
