use crate::{builder::Builder, log_entry::log_entry_protocol::LogEntryProtocol};
use std::{cell::Cell, fmt::Display, io, sync::Arc};
use tokio::sync::Mutex;

pub struct LogReader<T: LogEntryProtocol<T> + Display> {
    socket: Arc<Mutex<Cell<Option<tokio::net::TcpStream>>>>,
    entry_type: std::marker::PhantomData<T>,
}

impl<T: LogEntryProtocol<T> + Display> LogReader<T> {
    pub async fn new(builder: Builder<T>) -> io::Result<Self> {
        let socket = tokio::net::TcpStream::connect(builder.reader_addr).await?;
        socket.set_nodelay(true).ok();
        Ok(Self {
            socket: Arc::new(Mutex::new(Cell::new(Some(socket)))),
            entry_type: std::marker::PhantomData,
        })
    }

    pub async fn read(&self) -> io::Result<T> {
        let mg = self.socket.lock().await;
        let mut socket = mg.take().unwrap();
        let result = T::read_from(&mut socket).await;
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
