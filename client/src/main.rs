use common::{AuthMessage, Data, UserMessage, UserSID};
use futures::{SinkExt, StreamExt};
use tokio_tungstenite::{connect_async, tungstenite::Message};
use url::Url;

#[tokio::main]
async fn main() {
    let my_usid = UserSID { sid: 10 };
    let to_usid = UserSID { sid: 10 };
    let url = Url::parse(&format!("ws://127.0.0.1:3000/ws")).unwrap();

    let (mut ws_stream, _) = connect_async(url).await.expect("Failed to connect");

    // let (mut write_half, mut read_half) = ws_stream.split();

    println!("Connected to server");

    let auth_mes = AuthMessage { from: my_usid };

    ws_stream
        .send(Message::Binary(Data::Auth(auth_mes).into()))
        .await
        .unwrap();

    let mes = UserMessage {
        from: my_usid,
        to: to_usid,
        message: "Hello".to_string(),
    };

    ws_stream
        .send(Message::Binary(Data::UserMessage(mes).into()))
        .await
        .unwrap();

    if let Some(msg) = ws_stream.next().await {
        println!("Received: {:?}", msg);
    }

    ws_stream.close(None).await.unwrap();
}
