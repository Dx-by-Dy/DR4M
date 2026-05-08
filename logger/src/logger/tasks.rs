use crate::{builder::Builder, log_entry::log_entry_protocol::LogEntryProtocol};
use std::{collections::VecDeque, fmt::Display};
use tokio::{
    net::TcpListener,
    sync::{broadcast, mpsc, oneshot},
};

pub fn spawn_buffer_task<T: LogEntryProtocol<T> + Display>(
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

pub fn spawn_writer_task<T: LogEntryProtocol<T> + Display>(
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

pub fn spawn_reader_task<T: LogEntryProtocol<T> + Display>(
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
