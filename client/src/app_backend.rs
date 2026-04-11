use common::UserSID;
use futures::StreamExt;
use tokio_tungstenite::connect_async;
use url::Url;

use crate::Tx;

pub struct AppBackend {
    usid: Option<UserSID>,
    tx: Option<Tx>,
}

impl AppBackend {
    pub fn new() -> Self {
        Self {
            usid: None,
            tx: None,
        }
    }

    pub async fn set_usid(&mut self, usid: UserSID) {
        self.usid = Some(usid);
    }

    pub async fn connect(&mut self) {
        let (ws_stream, _) = connect_async(Url::parse(&format!("ws://127.0.0.1:3000/ws")).unwrap())
            .await
            .expect("Failed to connect");

        let (tx, mut rx) = ws_stream.split();
        self.tx = Some(tx);

        tokio::spawn(async move {
            while let Some(msg) = rx.next().await {
                println!("Received: {:?}", msg);
            }
        });
    }
}
