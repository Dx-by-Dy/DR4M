use crate::log_entry::LogEntry;

pub struct LogWriter {}

impl LogWriter {
    pub async fn write(&self, _le: LogEntry) {}
}
