pub mod builder;
pub mod log_entry;
pub mod log_reader;
pub mod log_writer;
pub mod logger;

#[macro_export]
macro_rules! async_log {
    ($value:expr) => {
        crate::LOGGER_WRITER.get().unwrap().write($value)
    };
}

// #[macro_export]
// macro_rules! sync_log {
//     ($value:expr) => {
//         _ = crate::LOGGER_WRITER.write($value).await;
//     };
// }

#[macro_export]
macro_rules! async_read_log {
    () => {
        crate::LOGGER_READER.get().unwrap().read()
    };
}

// #[macro_export]
// macro_rules! sync_read_log {
//     () => {
//         crate::LOG_RUNTIME.block_on(crate::LOGGER_READER.read())
//     };
// }
