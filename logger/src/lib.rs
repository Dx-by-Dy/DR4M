pub mod builder;
pub mod log_entry;
pub mod log_reader;
pub mod log_writer;
pub mod logger;

#[macro_export]
macro_rules! LOGGER_INIT {
    ($builder:expr, $ty:ty) => {
        static LOGGER_BUILDER: std::sync::LazyLock<logger::builder::Builder<$ty>> =
            std::sync::LazyLock::new(|| $builder);
        static LOG_RUNTIME: std::sync::LazyLock<tokio::runtime::Runtime> =
            std::sync::LazyLock::new(|| tokio::runtime::Runtime::new().unwrap());
        static LOGGER_READER: std::sync::LazyLock<logger::log_reader::LogReader<$ty>> =
            std::sync::LazyLock::new(|| {
                tokio::runtime::Runtime::new()
                    .unwrap()
                    .block_on(LOGGER_BUILDER.reader())
                    .unwrap()
            });
        static LOGGER_WRITER: std::sync::LazyLock<logger::log_writer::LogWriter<$ty>> =
            std::sync::LazyLock::new(|| {
                tokio::runtime::Runtime::new()
                    .unwrap()
                    .block_on(LOGGER_BUILDER.writer())
                    .unwrap()
            });
    };
}

#[macro_export]
macro_rules! async_log {
    ($value:expr) => {
        _ = crate::LOG_RUNTIME.spawn(crate::LOGGER_WRITER.write($value));
    };
}

#[macro_export]
macro_rules! sync_log {
    ($value:expr) => {
        _ = crate::LOG_RUNTIME.block_on(crate::LOGGER_WRITER.write($value));
    };
}

#[macro_export]
macro_rules! async_read_log {
    () => {
        crate::LOGGER_READER.read()
    };
}

#[macro_export]
macro_rules! sync_read_log {
    () => {
        crate::LOG_RUNTIME.block_on(crate::LOGGER_READER.read())
    };
}
