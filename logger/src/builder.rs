use crate::{
    log_entry::log_entry_protocol::LogEntryProtocol, log_reader::LogReader, log_writer::LogWriter,
    logger::Logger,
};
use std::{fmt::Display, io, net::SocketAddr};

#[derive(Clone, Copy)]
pub struct Builder<T: LogEntryProtocol<T> + Display> {
    pub writer_addr: SocketAddr,
    pub reader_addr: SocketAddr,
    pub buffer_capacity: usize,
    pub channel_capacity: usize,
    pub broadcast_capacity: usize,

    entry_type: std::marker::PhantomData<T>,
}

impl<T: LogEntryProtocol<T> + Display> Builder<T> {
    pub fn new() -> Self {
        Self {
            writer_addr: SocketAddr::from(([127, 0, 0, 1], 5555)),
            reader_addr: SocketAddr::from(([127, 0, 0, 1], 5556)),
            buffer_capacity: 10_000,
            channel_capacity: 10_000,
            broadcast_capacity: 15_000,
            entry_type: std::marker::PhantomData,
        }
    }

    pub fn writer_addr(mut self, addr: SocketAddr) -> Self {
        self.writer_addr = addr;
        self
    }

    pub fn reader_addr(mut self, addr: SocketAddr) -> Self {
        self.reader_addr = addr;
        self
    }

    pub fn buffer_capacity(mut self, capacity: usize) -> Self {
        self.buffer_capacity = capacity;
        self
    }

    pub fn channel_capacity(mut self, capacity: usize) -> Self {
        self.channel_capacity = capacity;
        self
    }

    pub fn broadcast_capacity(mut self, capacity: usize) -> Self {
        self.broadcast_capacity = capacity;
        self
    }

    pub fn logger(self) -> Logger<T> {
        Logger::new(self)
    }

    pub fn writer(self) -> impl Future<Output = io::Result<LogWriter<T>>> + Send {
        LogWriter::new(self)
    }

    pub fn reader(self) -> impl Future<Output = io::Result<LogReader<T>>> + Send {
        LogReader::new(self)
    }
}
