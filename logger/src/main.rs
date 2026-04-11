use logger::log_entry::LogEntry;
use std::collections::VecDeque;
use tokio::{
    net::TcpListener,
    sync::{broadcast, mpsc, oneshot},
};

const BUFFER_CAPACITY: usize = 10_000;
const CHANNEL_CAPACITY: usize = 100_000;
const BROADCAST_CAPACITY: usize = 10_000;

#[tokio::main]
async fn main() {
    let writer_listener = TcpListener::bind("127.0.0.1:5555").await.unwrap();
    let reader_listener = TcpListener::bind("127.0.0.1:5556").await.unwrap();

    let (mpsc_sender, mpsc_reciever) = mpsc::channel::<LogEntry>(CHANNEL_CAPACITY);
    let (tx_snapshot, rx_snapshot) = mpsc::channel::<oneshot::Sender<Vec<LogEntry>>>(1024);
    let (tx_broadcast, _) = broadcast::channel::<LogEntry>(BROADCAST_CAPACITY);

    tokio::spawn(buffer_task(
        mpsc_reciever,
        rx_snapshot,
        tx_broadcast.clone(),
    ));

    tokio::spawn(async move {
        loop {
            let (socket, _addr) = match writer_listener.accept().await {
                Ok((socket, _addr)) => (socket, _addr),
                Err(_) => todo!(),
            };
            let mpsc_sender = mpsc_sender.clone();

            tokio::spawn(handle_writer(socket, mpsc_sender));
        }
    });

    loop {
        let (socket, _addr) = match reader_listener.accept().await {
            Ok((socket, _addr)) => (socket, _addr),
            Err(_) => todo!(),
        };

        let broadcast_rx = tx_broadcast.subscribe();

        let (snapshot_tx, snapshot_rx) = oneshot::channel();
        match tx_snapshot.send(snapshot_tx).await {
            Ok(_) => {}
            Err(_) => continue,
        }

        tokio::spawn(handle_reader(socket, snapshot_rx, broadcast_rx));
    }
}

async fn buffer_task(
    mut mpsc_reciever: mpsc::Receiver<LogEntry>,
    mut rx_snapshot: mpsc::Receiver<oneshot::Sender<Vec<LogEntry>>>,
    tx_broadcast: broadcast::Sender<LogEntry>,
) {
    let mut buffer = VecDeque::with_capacity(BUFFER_CAPACITY);

    loop {
        tokio::select! {
            Some(entry) = mpsc_reciever.recv() => {
                if buffer.len() == BUFFER_CAPACITY {
                    buffer.pop_front();
                }

                buffer.push_back(entry);
                let _ = tx_broadcast.send(entry);
            }

            Some(resp) = rx_snapshot.recv() => {
                let snapshot: Vec<_> = buffer.iter().cloned().collect();
                let _ = resp.send(snapshot);
            }
        }
    }
}

// TODO: fix match error handling, currently just break the loop on any error, which will close the connection
async fn handle_writer(mut socket: tokio::net::TcpStream, mpsc_sender: mpsc::Sender<LogEntry>) {
    loop {
        let entry = match LogEntry::new_from(&mut socket).await {
            Ok(entry) => entry,
            Err(_) => break,
        };

        match mpsc_sender.try_send(entry) {
            Ok(_) => {}
            Err(_) => break,
        }
    }
}

// TODO: fix match error handling, currently just break the loop on any error, which will close the connection
async fn handle_reader(
    mut socket: tokio::net::TcpStream,
    snapshot_rx: oneshot::Receiver<Vec<LogEntry>>,
    mut broadcast_rx: broadcast::Receiver<LogEntry>,
) {
    let snapshot = match snapshot_rx.await {
        Ok(snapshot) => snapshot,
        Err(_) => return,
    };

    for entry in snapshot {
        match entry.write_to(&mut socket).await {
            Ok(_) => {}
            Err(_) => return,
        };
    }

    loop {
        match broadcast_rx.recv().await {
            Ok(entry) => {
                match entry.write_to(&mut socket).await {
                    Ok(_) => {}
                    Err(_) => return,
                };
            }
            Err(broadcast::error::RecvError::Lagged(_)) => {}
            Err(_) => break,
        }
    }
}
