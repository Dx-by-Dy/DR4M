use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[derive(Clone, Copy)]
pub struct LogEntry {
    pub ts: u64,
    pub level: u8,
    pub msg: [u8; 1024],
}

impl LogEntry {
    pub async fn write_to(&self, socket: &mut tokio::net::TcpStream) -> tokio::io::Result<()> {
        socket.write_u64(self.ts).await?;
        socket.write_u8(self.level).await?;
        socket.write_u32(1024).await?;
        socket.write_all(&self.msg).await?;
        Ok(())
    }

    pub async fn new_from(socket: &mut tokio::net::TcpStream) -> tokio::io::Result<LogEntry> {
        let ts = socket.read_u64().await?;
        let level = socket.read_u8().await?;
        let len = socket.read_u32().await? as usize;
        let mut msg = [0u8; 1024];
        socket.read_exact(&mut msg[..len]).await?;
        Ok(LogEntry { ts, level, msg })
    }
}
