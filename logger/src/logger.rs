use crate::log_entry::LogEntryProtocol;
use std::{collections::VecDeque, fmt::Display, io, net::SocketAddr};
use tokio::{
    net::TcpListener,
    sync::{broadcast, mpsc, oneshot},
};

pub struct LogWriter<T: LogEntryProtocol<T> + Display> {
    socket: tokio::net::TcpStream,
    entry_type: std::marker::PhantomData<T>,
}

impl<T: LogEntryProtocol<T> + Display> LogWriter<T> {
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

pub struct LogReader<T: LogEntryProtocol<T> + Display> {
    socket: tokio::net::TcpStream,
    entry_type: std::marker::PhantomData<T>,
}

impl<T: LogEntryProtocol<T> + Display> LogReader<T> {
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

pub struct Logger<T: LogEntryProtocol<T> + Display> {
    buffer_handler: tokio::task::JoinHandle<()>,
    writer_handler: tokio::task::JoinHandle<()>,
    reader_handler: tokio::task::JoinHandle<()>,

    entry_type: std::marker::PhantomData<T>,
}

impl<T: LogEntryProtocol<T> + Display> Logger<T> {
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

fn spawn_buffer_task<T: LogEntryProtocol<T> + Display>(
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

fn spawn_writer_task<T: LogEntryProtocol<T> + Display>(
    mpsc_sender: mpsc::Sender<T>,
    builder: Builder<T>,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let writer_listener = match TcpListener::bind(builder.writer_addr).await {
            Ok(listener) => listener,
            Err(e) => {
                eprintln!("Failed starting writer listener: {}", e);
                return;
            }
        };

        loop {
            let (mut socket, _addr) = match writer_listener.accept().await {
                Ok((socket, _addr)) => (socket, _addr),
                Err(e) => {
                    eprintln!("Failed accepting writer connection: {}", e);
                    continue;
                }
            };
            let mpsc_sender = mpsc_sender.clone();

            tokio::spawn(async move {
                loop {
                    let entry = match T::read_from(&mut socket).await {
                        Ok(entry) => entry,
                        Err(e) => {
                            eprintln!("Failed reading entry from writer: {}", e);
                            return;
                        }
                    };

                    match mpsc_sender.try_send(entry) {
                        Ok(_) => {}
                        Err(mpsc::error::TrySendError::Full(_)) => {
                            eprintln!("Failed sending entry to buffer: full");
                            continue;
                        }
                        Err(mpsc::error::TrySendError::Closed(_)) => {
                            eprintln!("Failed sending entry to buffer: closed");
                            return;
                        }
                    }
                }
            });
        }
    })
}

fn spawn_reader_task<T: LogEntryProtocol<T> + Display>(
    tx_broadcast: broadcast::Sender<T>,
    tx_snapshot: mpsc::Sender<oneshot::Sender<Vec<T>>>,
    builder: Builder<T>,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let reader_listener = match TcpListener::bind(builder.reader_addr).await {
            Ok(listener) => listener,
            Err(e) => {
                eprintln!("Failed starting reader listener: {}", e);
                return;
            }
        };

        loop {
            let (mut socket, _addr) = match reader_listener.accept().await {
                Ok((socket, _addr)) => (socket, _addr),
                Err(e) => {
                    eprintln!("Failed accepting reader connection: {}", e);
                    continue;
                }
            };
            socket.set_nodelay(true).ok();

            let mut broadcast_rx = tx_broadcast.subscribe();

            let (snapshot_tx, snapshot_rx) = oneshot::channel();
            match tx_snapshot.send(snapshot_tx).await {
                Ok(_) => {}
                Err(e) => {
                    eprintln!("Failed sending snapshot: {}", e);
                    continue;
                }
            }

            tokio::spawn(async move {
                let snapshot = match snapshot_rx.await {
                    Ok(snapshot) => snapshot,
                    Err(e) => {
                        eprintln!("Failed receiving snapshot: {}", e);
                        return;
                    }
                };

                for entry in snapshot {
                    match entry.write_to(&mut socket).await {
                        Ok(_) => {}
                        Err(e) => {
                            eprintln!("Failed writing entry [{}]: {}", entry, e);
                            continue;
                        }
                    };
                }

                loop {
                    match broadcast_rx.recv().await {
                        Ok(entry) => {
                            match entry.write_to(&mut socket).await {
                                Ok(_) => {}
                                Err(e) => {
                                    eprintln!("Failed writing entry [{}]: {}", entry, e);
                                    continue;
                                }
                            };
                        }
                        Err(e) => match e {
                            broadcast::error::RecvError::Lagged(_) => {
                                eprintln!("Failed receiving broadcast entry: {}", e);
                                continue;
                            }
                            broadcast::error::RecvError::Closed => {
                                eprintln!("Broadcast channel closed");
                                return;
                            }
                        },
                    }
                }
            });
        }
    })
}

impl<T: LogEntryProtocol<T> + Display> Drop for Logger<T> {
    fn drop(&mut self) {
        self.reader_handler.abort();
        self.writer_handler.abort();
        self.buffer_handler.abort();
    }
}

#[derive(Clone, Copy)]
pub struct Builder<T: LogEntryProtocol<T> + Display> {
    writer_addr: SocketAddr,
    reader_addr: SocketAddr,
    buffer_capacity: usize,
    channel_capacity: usize,
    broadcast_capacity: usize,

    entry_type: std::marker::PhantomData<T>,
}

impl<T: LogEntryProtocol<T> + Display> Builder<T> {
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
