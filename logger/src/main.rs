use logger::{builder::Builder, log_entry::LogEntry};

#[tokio::main]
async fn main() {
    let builder = Builder::<LogEntry>::new();
    let logger = builder.logger();
    _ = logger.wait().await;
}
