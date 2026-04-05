use axum::{
    extract::{
        State, WebSocketUpgrade,
        ws::{Message, WebSocket},
    },
    response::IntoResponse,
};
use common::{AuthMessage, Data, UserMessage};
use futures::{SinkExt, StreamExt};

use crate::{clients::Clients, server_user_state::ServerUserState};

pub async fn ws_handler(ws: WebSocketUpgrade, State(clients): State<Clients>) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, clients))
}

async fn handle_socket(socket: WebSocket, mut clients: Clients) {
    let (tx, mut rx) = socket.split();
    let mut state = ServerUserState {
        usid: None,
        tx: Some(tx),
    };

    tokio::spawn(async move {
        loop {
            let Some(output) = rx.next().await else {
                break;
            };
            match output {
                Ok(Message::Binary(bin)) => match bin.into() {
                    Data::Auth(auth_message) => {
                        handle_auth_message(auth_message, &mut clients, &mut state).await;
                    }
                    Data::UserMessage(message) => {
                        handle_user_message(message, &mut clients).await;
                    }
                },
                Ok(Message::Close(_)) => {
                    handle_close(&mut clients, &mut state).await;
                    break;
                }
                Ok(Message::Text(_)) | Ok(Message::Ping(_)) | Ok(Message::Pong(_)) => {}
                Err(e) => {
                    handle_close(&mut clients, &mut state).await;
                    // TODO: fix error handling
                    println!("Error in WebSocket connection: {:?}", e);
                    break;
                }
            }
        }
    });
}

async fn handle_user_message(message: UserMessage, clients: &mut Clients) {
    clients
        .send(
            message.to,
            Message::Binary(Data::UserMessage(message).into()),
        )
        .await
}

async fn handle_auth_message(
    message: AuthMessage,
    clients: &mut Clients,
    state: &mut ServerUserState,
) {
    if let Some(tx) = state.tx.take() {
        state.usid = Some(message.from);
        clients.auth(message.from, tx).await;
    } else {
        todo!();
    }
}

async fn handle_close(clients: &mut Clients, state: &mut ServerUserState) {
    if let Some(mut tx) = state.tx.take() {
        tx.close().await.unwrap();
    } else {
        clients.close(state.usid.unwrap()).await;
    }
}
