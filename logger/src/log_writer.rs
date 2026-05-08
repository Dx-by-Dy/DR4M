use crate::{builder::Builder, log_entry::log_entry_protocol::LogEntryProtocol};
use std::{cell::Cell, fmt::Display, io, sync::Arc};
use tokio::sync::Mutex;

pub struct LogWriter<T: LogEntryProtocol<T> + Display> {
    socket: Arc<Mutex<Cell<Option<tokio::net::TcpStream>>>>,
    entry_type: std::marker::PhantomData<T>,
}

impl<T: LogEntryProtocol<T> + Display> LogWriter<T> {
    pub async fn new(builder: Builder<T>) -> io::Result<Self> {
        let socket = tokio::net::TcpStream::connect(builder.writer_addr).await?;
        socket.set_nodelay(true).ok();
        Ok(Self {
            socket: Arc::new(Mutex::new(Cell::new(Some(socket)))),
            entry_type: std::marker::PhantomData,
        })
    }

    pub async fn write(&self, entry: T) -> io::Result<()> {
        let mg = self.socket.lock().await;
        let mut socket = mg.take().unwrap();
        let result = entry.write_to(&mut socket).await;
        mg.set(Some(socket));
        result
    }

    pub fn clone(&self) -> Self {
        Self {
            socket: self.socket.clone(),
            entry_type: std::marker::PhantomData,
        }
    }
}
