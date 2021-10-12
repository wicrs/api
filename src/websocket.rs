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
        let (mut websocket, _) = connect_async(request).await?;
        websocket.send(Message::Text(user_id.to_string())).await?;
        Ok(Self { user_id, websocket })
    }

    pub async fn event_loop<F, R>(self, action: F) -> Result<R>
    where
        F: Fn(Arc<Mutex<Self>>, ServerMessage) -> Option<R>,
    {
        let arc = Arc::new(Mutex::new(self));
        loop {
            let message = {
                let arc = Arc::clone(&arc);
                let mut lock = arc.lock().await;
                lock.next_ws_message().await?
            };
            if let Some(r) = action(Arc::clone(&arc), message) {
                return Ok(r);
            }
        }
    }

    pub async fn next_ws_message(&mut self) -> Result<ServerMessage> {
        if let Some(message) = self.websocket.next().await {
            let message = message?;
            let text = message.to_text()?;
            Ok(serde_json::from_str(text)?)
        } else {
            Err(Error::WsClosed)
        }
    }

    pub async fn send_ws_message(&mut self, message: ClientMessage) -> Result<()> {
        self.websocket
            .send(Message::Text(serde_json::to_string(&message)?))
            .await?;
        self.websocket.flush().await?;
        Ok(())
    }

    pub async fn subscribe_hub(&mut self, hub_id: ID) -> Result<()> {
        self.send_ws_message(ClientMessage::SubscribeHub { hub_id })
            .await
    }

    pub async fn subscribe_channel(&mut self, hub_id: ID, channel_id: ID) -> Result<()> {
        self.send_ws_message(ClientMessage::SubscribeChannel { hub_id, channel_id })
            .await
    }

    pub async fn unsubscribe_hub(&mut self, hub_id: ID) -> Result<()> {
        self.send_ws_message(ClientMessage::UnsubscribeHub { hub_id })
            .await
    }

    pub async fn unsubscribe_channel(&mut self, hub_id: ID, channel_id: ID) -> Result<()> {
        self.send_ws_message(ClientMessage::UnsubscribeChannel { hub_id, channel_id })
            .await
    }
}