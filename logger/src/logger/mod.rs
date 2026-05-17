mod tasks;

use crate::{
    builder::Builder,
    log_entry::log_entry_protocol::LogEntryProtocol,
    logger::tasks::{spawn_buffer_task, spawn_reader_task, spawn_writer_task},
};
use std::fmt::Display;
use tokio::{
    join,
    sync::{broadcast, mpsc, oneshot},
    task::JoinError,
};

pub struct Logger<T: LogEntryProtocol<T> + Display> {
    buffer_handler: tokio::task::JoinHandle<()>,
    writer_handler: tokio::task::JoinHandle<()>,
    reader_handler: tokio::task::JoinHandle<()>,

    entry_type: std::marker::PhantomData<T>,
}

impl<T: LogEntryProtocol<T> + Display> Logger<T> {
    pub fn new(builder: Builder<T>) -> Self {
        let (mpsc_sender, mpsc_reciever) = mpsc::channel::<T>(builder.channel_capacity);
        let (tx_snapshot, rx_snapshot) = mpsc::channel::<oneshot::Sender<Vec<T>>>(1024);
        let (tx_broadcast, _) = broadcast::channel::<T>(builder.broadcast_capacity);
        let buffer_tx_broadcast = tx_broadcast.clone();

        let buffer_handler =
            spawn_buffer_task(mpsc_reciever, rx_snapshot, buffer_tx_broadcast, builder);
        let writer_handler = spawn_writer_task(mpsc_sender, builder);
        let reader_handler = spawn_reader_task(tx_broadcast, tx_snapshot, builder);

        Logger {
            buffer_handler,
            writer_handler,
            reader_handler,
            entry_type: std::marker::PhantomData,
        }
    }

    pub async fn wait(
        self,
    ) -> (
        Result<(), JoinError>,
        Result<(), JoinError>,
        Result<(), JoinError>,
    ) {
        join!(
            self.buffer_handler,
            self.writer_handler,
            self.reader_handler
        )
    }
}
