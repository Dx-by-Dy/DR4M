//pub mod app_backend;
pub mod chat;
pub mod commander;
pub mod connection;
pub mod controller;
pub mod inputter;
pub mod log_reader;
pub mod ui;

pub type Tx = futures::stream::SplitSink<
    tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>,
    tokio_tungstenite::tungstenite::Message,
>;

pub static LOGGER_READER: tokio::sync::OnceCell<
    logger::log_reader::LogReader<logger::log_entry::LogEntry>,
> = tokio::sync::OnceCell::const_new();
pub static LOGGER_WRITER: tokio::sync::OnceCell<
    logger::log_writer::LogWriter<logger::log_entry::LogEntry>,
> = tokio::sync::OnceCell::const_new();
