use logger::{log_entry::LogEntry, logger::Builder};

#[tokio::main]
async fn main() {
    let builder = Builder::<LogEntry>::new();
    let logger = builder.logger();
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    let mut writer = builder.writer().await.unwrap();
    let mut reader = builder.reader().await.unwrap();

    let mut msg: [u8; 1024] = [0; 1024];
    let slice_msg = b"Hello";
    msg[..slice_msg.len()].copy_from_slice(slice_msg);
    writer
        .write(LogEntry {
            ts: 123456789,
            level: 1,
            len: slice_msg.len() as u16,
            msg,
        })
        .await
        .unwrap();

    let entry = reader.read().await.unwrap();

    println!(
        "LogEntry {{ ts={}, level={}, len={}, msg={:?} }}",
        entry.ts,
        entry.level,
        entry.len,
        &entry.msg[..entry.len as usize]
    );

    let mut msg: [u8; 1024] = [0; 1024];
    let slice_msg = b"Hello?";
    msg[..slice_msg.len()].copy_from_slice(slice_msg);
    writer
        .write(LogEntry {
            ts: 123456789,
            level: 1,
            len: slice_msg.len() as u16,
            msg,
        })
        .await
        .unwrap();

    drop(logger);
}
