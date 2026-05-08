pub trait LogEntryProtocol<T: std::fmt::Display>: Clone + Copy + Send + 'static {
    fn write_to(
        self,
        socket: &mut tokio::net::TcpStream,
    ) -> impl Future<Output = Result<(), std::io::Error>> + Send;
    fn read_from(
        socket: &mut tokio::net::TcpStream,
    ) -> impl Future<Output = tokio::io::Result<T>> + Send;
}
