use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct LogEntry {
    pub len: u16,
    pub msg: [u8; 1024],
}

pub trait LogEntryProtocol<T: std::fmt::Display>: Clone + Copy + Send + 'static {
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
            let mut buf = [0u8; 1026];
            buf[..2].copy_from_slice(&self.len.to_be_bytes());
            buf[2..2 + self.len as usize].copy_from_slice(&self.msg[..self.len as usize]);
            socket.write_all(&buf[..2 + self.len as usize]).await?;
            socket.flush().await
        }
    }

    fn read_from(
        socket: &mut tokio::net::TcpStream,
    ) -> impl Future<Output = tokio::io::Result<LogEntry>> + Send {
        async move {
            let len = socket.read_u16().await?;
            let mut msg = [0u8; 1024];
            let readed_len = socket.read_exact(&mut msg[..len as usize]).await?;
            if readed_len != len as usize {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Failed reading log entry",
                ));
            }
            Ok(LogEntry { len, msg })
        }
    }
}

impl std::fmt::Display for LogEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            String::from_utf8_lossy(&self.msg[..self.len as usize])
        )
    }
}

impl From<&[u8]> for LogEntry {
    fn from(s: &[u8]) -> Self {
        let mut msg = [0u8; 1024];
        let len = s.len().min(1024);
        msg[..len].copy_from_slice(&s[..len]);
        LogEntry {
            len: len as u16,
            msg,
        }
    }
}
