use axum::extract::ws::Message;
use common::UserSID;
use futures::SinkExt;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex;

use crate::Tx;

#[derive(Default, Clone)]
pub struct Clients {
    // TODO: fix global lock
    clients: Arc<Mutex<HashMap<UserSID, Tx>>>,
}

impl Clients {
    pub async fn auth(&mut self, usid: UserSID, mut tx: Tx) {
        let mut m = self.clients.lock().await;
        if !m.contains_key(&usid) {
            m.insert(usid, tx);
        } else {
            // TODO: fix unwrap()
            tx.close().await.unwrap();
        }
    }

    pub async fn send(&mut self, usid: UserSID, message: Message) {
        let mut m = self.clients.lock().await;
        if let Some(tx) = m.get_mut(&usid) {
            tx.send(message).await.unwrap();
        } else {
            todo!();
        }
    }

    pub async fn close(&mut self, usid: UserSID) {
        let mut m = self.clients.lock().await;
        let mut tx = m.remove(&usid).unwrap();
        tx.close().await.unwrap();
    }
}
