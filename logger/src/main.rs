use logger::{builder::Builder, log_entry::LogEntry};

#[tokio::main]
async fn main() {
    let builder = Builder::<LogEntry>::new();
    let logger = builder.logger();
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    let writer = builder.writer().await.unwrap();
    let reader = builder.reader().await.unwrap();

    writer
        .write(LogEntry::from("Hello".as_bytes()))
        .await
        .unwrap();

    let n_writer = writer.clone();
    n_writer
        .write(LogEntry::from("Hello?".as_bytes()))
        .await
        .unwrap();

    println!("{}", reader.read().await.unwrap());

    writer
        .write(LogEntry::from("Hello!".as_bytes()))
        .await
        .unwrap();

    println!("{}", reader.read().await.unwrap());
    println!("{}", reader.read().await.unwrap());

    drop(logger);
}
