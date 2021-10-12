use std::sync::Arc;

use crate::{error::Result, Error};
use futures_util::{SinkExt, StreamExt};
use tokio::{net::TcpStream, sync::Mutex};
use tokio_tungstenite::tungstenite::handshake::client::Request;
use tokio_tungstenite::{connect_async, tungstenite::Message, MaybeTlsStream, WebSocketStream};
use wicrs_server::{
    websocket::{ClientMessage, ServerMessage},
    ID,
};

pub struct WebsocketClient {
    pub user_id: ID,
    websocket: WebSocketStream<MaybeTlsStream<TcpStream>>,
}

impl WebsocketClient {
    pub async fn new(user_id: ID, server_api_url: &str) -> Result<Self> {
        let request = Request::builder()
            .uri(&format!("{}/websocket", server_api_url))
            .header("authorization", &user_id.to_string())
            .body(())
            .unwrap();
        let (websocket, _) = connect_async(request).await?;
        Ok(Self { user_id, websocket })
    }

    pub async fn event_loop<F, R>(self, action: F) -> Result<R>
    where
        F: Fn(Arc<Mutex<Self>>, ServerMessage) -> Option<R>,
    {
        println!("start loop!");
        let arc = Arc::new(Mutex::new(self));
        loop {
            println!("loop");
            let message = {
                let arc = Arc::clone(&arc);
                println!("await lock");
                let mut lock = arc.lock().await;
                println!("locked");
                lock.next_message().await?
            };
            if let Some(r) = action(Arc::clone(&arc), message) {
                return Ok(r);
            }
        }
    }

    pub async fn next_message(&mut self) -> Result<ServerMessage> {
        if let Some(message) = dbg!(self.websocket.next().await) {
            let message = message?;
            let text = message.to_text()?;
            Ok(serde_json::from_str(text)?)
        } else {
            Err(Error::WsClosed)
        }
    }

    pub async fn send_message(&mut self, message: ClientMessage) -> Result<()> {
        self.websocket
            .send(Message::Text(serde_json::to_string(&message)?))
            .await?;
        self.websocket.flush().await?;
        Ok(())
    }
}
