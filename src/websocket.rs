use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use crate::{error::Result, Error};
use futures_util::stream::{SplitSink, SplitStream};
use futures_util::{SinkExt, StreamExt};
use tokio::{net::TcpStream, sync::Mutex};
use tokio_tungstenite::tungstenite::handshake::client::Request;
use tokio_tungstenite::{connect_async, tungstenite::Message, MaybeTlsStream, WebSocketStream};
use wicrs_server::{
    websocket::{ClientMessage, ServerMessage},
    ID,
};

use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};

pub struct WebsocketClient {
    pub user_id: ID,
    websocket_send: Mutex<SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>>,
    websocket_recv: Mutex<SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>>,
    sender: Mutex<UnboundedSender<Result<()>>>,
    receiver: Mutex<UnboundedReceiver<Result<()>>>,
    loop_running: AtomicBool,
}

impl WebsocketClient {
    pub async fn new(user_id: ID, server_api_url: &str) -> Result<Arc<Self>> {
        let request = Request::builder()
            .uri(&format!("{}/websocket", server_api_url))
            .header("authorization", &user_id.to_string())
            .body(())
            .unwrap();
        let (websocket, _) = connect_async(request).await?;
        let (mut s, r) = websocket.split();
        s.send(Message::Text(user_id.to_string())).await?;
        let (send, recv) = unbounded_channel();
        Ok(Arc::new(Self {
            user_id,
            websocket_send: Mutex::new(s),
            websocket_recv: Mutex::new(r),
            sender: Mutex::new(send),
            receiver: Mutex::new(recv),
            loop_running: AtomicBool::new(false),
        }))
    }

    pub async fn start_loop<F, R>(self: Arc<Self>, action: F) -> Result<R>
    where
        F: Fn(Arc<Self>, ServerMessage) -> Option<R>,
    {
        let sender = self.sender.lock().await;
        self.loop_running.store(true, Ordering::Release);
        let message_loop = || async {
            loop {
                let message = self.next_ws_message().await?;
                match message {
                    ServerMessage::Success => {
                        sender.send(Ok(())).map_err(|_| Error::TokioMpscSend)?;
                    }
                    ServerMessage::Error(e) => {
                        sender
                            .send(Err(wicrs_server::error::Error::from(e).into()))
                            .map_err(|_| Error::TokioMpscSend)?;
                    }
                    _ => {
                        if let Some(r) = action(Arc::clone(&self), message) {
                            return Ok(r);
                        }
                    }
                }
            }
        };
        let result = message_loop().await;
        self.loop_running.store(false, Ordering::Release);
        result
    }

    async fn next_ws_message(&self) -> Result<ServerMessage> {
        if let Some(message) = {
            let mut lock = self.websocket_recv.lock().await;
            lock.next().await
        } {
            let message = message?;
            let text = message.to_text()?;
            Ok(serde_json::from_str(text)?)
        } else {
            Err(Error::WsClosed)
        }
    }

    /// Sends a message to the server over websocket, if self.sender is not locked, do not wait for response...
    async fn send_ws_message(&self, message: ClientMessage) -> Result<()> {
        let mut receiver = self.receiver.lock().await;
        {
            let mut lock = self.websocket_send.lock().await;
            lock.send(Message::Text(serde_json::to_string(&message)?))
                .await?;
            lock.flush().await?;
        }

        if self.loop_running.load(Ordering::Acquire) {
            if let Some(result) = receiver.recv().await {
                result
            } else {
                Err(Error::TokioMpscSend)
            }
        } else {
            Ok(())
        }
    }

    pub async fn send_message(&self, hub_id: ID, channel_id: ID, message: String) -> Result<()> {
        self.send_ws_message(ClientMessage::SendMessage {
            hub_id,
            channel_id,
            message,
        })
        .await
    }

    pub async fn subscribe_hub(&self, hub_id: ID) -> Result<()> {
        self.send_ws_message(ClientMessage::SubscribeHub { hub_id })
            .await
    }

    pub async fn subscribe_channel(&self, hub_id: ID, channel_id: ID) -> Result<()> {
        self.send_ws_message(ClientMessage::SubscribeChannel { hub_id, channel_id })
            .await
    }

    pub async fn unsubscribe_hub(&self, hub_id: ID) -> Result<()> {
        self.send_ws_message(ClientMessage::UnsubscribeHub { hub_id })
            .await
    }

    pub async fn unsubscribe_channel(&self, hub_id: ID, channel_id: ID) -> Result<()> {
        self.send_ws_message(ClientMessage::UnsubscribeChannel { hub_id, channel_id })
            .await
    }

    pub async fn start_typing(&self, hub_id: ID, channel_id: ID) -> Result<()> {
        self.send_ws_message(ClientMessage::StartTyping { hub_id, channel_id })
            .await
    }

    pub async fn stop_typing(&self, hub_id: ID, channel_id: ID) -> Result<()> {
        self.send_ws_message(ClientMessage::StopTyping { hub_id, channel_id })
            .await
    }
}
