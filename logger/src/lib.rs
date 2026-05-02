pub mod log_entry;
pub mod logger;

#[macro_export]
macro_rules! LOGGER_INIT {
    ($builder:expr, $ty:ty) => {
        static LOGGER_BUILDER: std::sync::LazyLock<logger::logger::Builder<$ty>> =
            std::sync::LazyLock::new(|| $builder);
        static SYNC_LOG_RUNTIME: std::sync::LazyLock<tokio::runtime::Runtime> =
            std::sync::LazyLock::new(|| tokio::runtime::Runtime::new().unwrap());
        static LOGGER_READER: std::sync::LazyLock<logger::logger::LogReader<$ty>> =
            std::sync::LazyLock::new(|| {
                tokio::runtime::Runtime::new()
                    .unwrap()
                    .block_on(LOGGER_BUILDER.reader())
                    .unwrap()
            });
        static LOGGER_WRITER: std::sync::LazyLock<logger::logger::LogWriter<$ty>> =
            std::sync::LazyLock::new(|| {
                tokio::runtime::Runtime::new()
                    .unwrap()
                    .block_on(LOGGER_BUILDER.writer())
                    .unwrap()
            });
    };
}

#[macro_export]
macro_rules! async_log_quiet {
    ($value:expr) => {
        _ = crate::LOGGER_WRITER.write($value).await;
    };
}

#[macro_export]
macro_rules! sync_log_quiet {
    ($value:expr) => {
        _ = crate::SYNC_LOG_RUNTIME.block_on(crate::LOGGER_WRITER.write($value));
    };
}

#[macro_export]
macro_rules! async_log {
    ($value:expr) => {
        crate::LOGGER_WRITER.write($value).await
    };
}

#[macro_export]
macro_rules! sync_log {
    ($value:expr) => {
        crate::SYNC_LOG_RUNTIME.block_on(crate::LOGGER_WRITER.write($value))
    };
}
