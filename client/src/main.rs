use dr4m_client::{
    LOGGER_READER, LOGGER_WRITER,
    connection::Connection,
    controller::{Controller, component::Component},
};

#[tokio::main]
async fn main() {
    _ = LOGGER_READER.set(
        logger::builder::Builder::<logger::log_entry::LogEntry>::new()
            .reader()
            .await
            .unwrap(),
    );
    _ = LOGGER_WRITER.set(
        logger::builder::Builder::<logger::log_entry::LogEntry>::new()
            .writer()
            .await
            .unwrap(),
    );
    _ = Controller::start(Connection::new()).await;
}
