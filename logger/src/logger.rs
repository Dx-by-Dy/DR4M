use crate::log_entry::LogEntryProtocol;
use std::{collections::VecDeque, io, net::SocketAddr};
use tokio::{
    net::TcpListener,
    sync::{broadcast, mpsc, oneshot},
};

pub struct LogWriter<T: LogEntryProtocol<T>> {
    socket: tokio::net::TcpStream,
    entry_type: std::marker::PhantomData<T>,
}

impl<T: LogEntryProtocol<T>> LogWriter<T> {
    pub async fn new(builder: Builder<T>) -> io::Result<Self> {
        let socket = tokio::net::TcpStream::connect(builder.writer_addr).await?;
        socket.set_nodelay(true).ok();
        Ok(Self {
            socket,
            entry_type: std::marker::PhantomData,
        })
    }

    pub async fn write(&mut self, entry: T) -> io::Result<()> {
        entry.write_to(&mut self.socket).await
    }

    pub async fn try_clone(&self) -> io::Result<Self> {
        let socket = tokio::net::TcpStream::connect(self.socket.peer_addr()?).await?;
        Ok(Self {
            socket,
            entry_type: std::marker::PhantomData,
        })
    }
}

pub struct LogReader<T: LogEntryProtocol<T>> {
    socket: tokio::net::TcpStream,
    entry_type: std::marker::PhantomData<T>,
}

impl<T: LogEntryProtocol<T>> LogReader<T> {
    pub async fn new(builder: Builder<T>) -> io::Result<Self> {
        let socket = tokio::net::TcpStream::connect(builder.reader_addr).await?;
        socket.set_nodelay(true).ok();
        Ok(Self {
            socket,
            entry_type: std::marker::PhantomData,
        })
    }

    pub async fn read(&mut self) -> io::Result<T> {
        T::read_from(&mut self.socket).await
    }

    pub async fn try_clone(&self) -> io::Result<Self> {
        let socket = tokio::net::TcpStream::connect(self.socket.peer_addr()?).await?;
        Ok(Self {
            socket,
            entry_type: std::marker::PhantomData,
        })
    }
}

pub struct Logger<T: LogEntryProtocol<T>> {
    buffer_handler: tokio::task::JoinHandle<()>,
    writer_handler: tokio::task::JoinHandle<()>,
    reader_handler: tokio::task::JoinHandle<()>,

    entry_type: std::marker::PhantomData<T>,
}

impl<T: LogEntryProtocol<T>> Logger<T> {
    fn new(builder: Builder<T>) -> Self {
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
}

fn spawn_buffer_task<T: LogEntryProtocol<T>>(
    mut mpsc_reciever: mpsc::Receiver<T>,
    mut rx_snapshot: mpsc::Receiver<oneshot::Sender<Vec<T>>>,
    buffer_tx_broadcast: broadcast::Sender<T>,
    builder: Builder<T>,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let mut buffer = VecDeque::with_capacity(builder.buffer_capacity);

        loop {
            tokio::select! {
                Some(entry) = mpsc_reciever.recv() => {
                    if buffer.len() == builder.buffer_capacity {
                        buffer.pop_front();
                    }

                    buffer.push_back(entry);
                    let _ = buffer_tx_broadcast.send(entry);
                }

                Some(resp) = rx_snapshot.recv() => {
                    let snapshot = buffer.iter().cloned().collect();
                    let _ = resp.send(snapshot);
                }
            }
        }
    })
}

fn spawn_writer_task<T: LogEntryProtocol<T>>(
    mpsc_sender: mpsc::Sender<T>,
    builder: Builder<T>,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let writer_listener = match TcpListener::bind(builder.writer_addr).await {
            Ok(listener) => listener,
            Err(_) => unimplemented!(),
        };

        loop {
            let (mut socket, _addr) = match writer_listener.accept().await {
                Ok((socket, _addr)) => (socket, _addr),
                Err(_) => unimplemented!(),
            };
            let mpsc_sender = mpsc_sender.clone();

            tokio::spawn(async move {
                loop {
                    let entry = match T::read_from(&mut socket).await {
                        Ok(entry) => entry,
                        Err(_) => unimplemented!(),
                    };

                    match mpsc_sender.try_send(entry) {
                        Ok(_) => {}
                        Err(_) => unimplemented!(),
                    }
                }
            });
        }
    })
}

fn spawn_reader_task<T: LogEntryProtocol<T>>(
    tx_broadcast: broadcast::Sender<T>,
    tx_snapshot: mpsc::Sender<oneshot::Sender<Vec<T>>>,
    builder: Builder<T>,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let reader_listener = match TcpListener::bind(builder.reader_addr).await {
            Ok(listener) => listener,
            Err(_) => unimplemented!(),
        };

        loop {
            let (mut socket, _addr) = match reader_listener.accept().await {
                Ok((socket, _addr)) => (socket, _addr),
                Err(_) => unimplemented!(),
            };
            socket.set_nodelay(true).ok();

            let mut broadcast_rx = tx_broadcast.subscribe();

            let (snapshot_tx, snapshot_rx) = oneshot::channel();
            match tx_snapshot.send(snapshot_tx).await {
                Ok(_) => {}
                Err(_) => unimplemented!(),
            }

            tokio::spawn(async move {
                let snapshot = match snapshot_rx.await {
                    Ok(snapshot) => snapshot,
                    Err(_) => unimplemented!(),
                };

                for entry in snapshot {
                    match entry.write_to(&mut socket).await {
                        Ok(_) => {}
                        Err(_) => unimplemented!(),
                    };
                }

                loop {
                    match broadcast_rx.recv().await {
                        Ok(entry) => {
                            match entry.write_to(&mut socket).await {
                                Ok(_) => {}
                                Err(_) => unimplemented!(),
                            };
                        }
                        Err(broadcast::error::RecvError::Lagged(_)) => {}
                        Err(_) => unimplemented!(),
                    }
                }
            });
        }
    })
}

impl<T: LogEntryProtocol<T>> Drop for Logger<T> {
    fn drop(&mut self) {
        self.reader_handler.abort();
        self.writer_handler.abort();
        self.buffer_handler.abort();
    }
}

#[derive(Clone, Copy)]
pub struct Builder<T: LogEntryProtocol<T>> {
    writer_addr: SocketAddr,
    reader_addr: SocketAddr,
    buffer_capacity: usize,
    channel_capacity: usize,
    broadcast_capacity: usize,

    entry_type: std::marker::PhantomData<T>,
}

impl<T: LogEntryProtocol<T>> Builder<T> {
    pub fn new() -> Self {
        Self {
            writer_addr: SocketAddr::from(([127, 0, 0, 1], 5555)),
            reader_addr: SocketAddr::from(([127, 0, 0, 1], 5556)),
            buffer_capacity: 10_000,
            channel_capacity: 100_000,
            broadcast_capacity: 10_000,
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
