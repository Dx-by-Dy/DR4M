use tokio::task::JoinHandle;

use crate::connection::Connection;

pub trait Component: Sized {
    fn new(connection: Connection) -> Self;
    fn main_loop(self) -> impl Future<Output = ()> + Send + 'static;
    fn start(connection: Connection) -> JoinHandle<()> {
        tokio::spawn(Self::new(connection).main_loop())
    }
}
