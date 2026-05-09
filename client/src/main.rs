use dr4m_client::{
    connection::Connection,
    controller::{Controller, component::Component},
};

#[tokio::main]
async fn main() {
    _ = Controller::start(Connection::new()).await;
}
