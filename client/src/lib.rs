//pub mod app_backend;
//pub mod chat_manager;
pub mod connection;
pub mod controller;
pub mod inputter;
pub mod log_reader;
pub mod ui;

pub type Tx = futures::stream::SplitSink<
    tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>,
    tokio_tungstenite::tungstenite::Message,
>;

logger::LOGGER_INIT!(
    logger::builder::Builder::<logger::log_entry::LogEntry>::new(),
    logger::log_entry::LogEntry
);
