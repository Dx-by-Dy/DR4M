use crate::connection::Connection;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;

pub trait Component: Sized {
    fn new(connection: Connection) -> Self;
    fn main_loop(self) -> impl Future<Output = ()> + Send + 'static;
    fn start(connection: Connection) -> JoinHandle<()> {
        tokio::spawn(Self::new(connection).main_loop())
    }
}

pub trait Quit {
    fn cancellation_token(&self) -> &CancellationToken;
}
