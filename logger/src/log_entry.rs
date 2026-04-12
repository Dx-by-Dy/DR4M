use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[derive(Debug, Clone, Copy)]
pub struct LogEntry {
    pub ts: u64,
    pub level: u8,
    pub len: u16,
    pub msg: [u8; 1024],
}

pub trait LogEntryProtocol<T>: Clone + Copy + Send + 'static {
    fn write_to(
        self,
        socket: &mut tokio::net::TcpStream,
    ) -> impl Future<Output = Result<(), std::io::Error>> + Send;
    fn read_from(
        socket: &mut tokio::net::TcpStream,
    ) -> impl Future<Output = tokio::io::Result<T>> + Send;
}

impl LogEntryProtocol<LogEntry> for LogEntry {
    fn write_to(
        self,
        socket: &mut tokio::net::TcpStream,
    ) -> impl Future<Output = Result<(), std::io::Error>> + Send {
        async move {
            let mut buf = [0u8; 1035];
            buf[..8].copy_from_slice(&self.ts.to_be_bytes());
            buf[8] = self.level;
            buf[9..11].copy_from_slice(&self.len.to_be_bytes());
            buf[11..11 + self.len as usize].copy_from_slice(&self.msg[..self.len as usize]);
            socket.write_all(&buf[..11 + self.len as usize]).await?;
            socket.flush().await
        }
    }

    fn read_from(
        socket: &mut tokio::net::TcpStream,
    ) -> impl Future<Output = tokio::io::Result<LogEntry>> + Send {
        async move {
            let ts = socket.read_u64().await?;
            let level = socket.read_u8().await?;
            let len = socket.read_u16().await?;
            let mut msg = [0u8; 1024];
            socket.read_exact(&mut msg[..len as usize]).await?;
            Ok(LogEntry {
                ts,
                level,
                len,
                msg,
            })
        }
    }
}
