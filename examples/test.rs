extern crate wicrs_api;

use wicrs_api::{error::Result, http::HttpClient, websocket::WebsocketClient};
use wicrs_server::websocket::ClientMessage;

#[tokio::main]
pub async fn main() -> Result<()> {
    let server_api_url = "http://localhost:8080/api".to_string();
    let user_one = wicrs_server::new_id();
    let user_two = wicrs_server::new_id();
    let client_one = HttpClient::new(user_one, server_api_url.clone())?;
    let hub_id = client_one.hub_create("test".to_string()).await?;
    let hub = client_one.hub_get(hub_id).await?;
    let channel_id = *hub.channels.keys().next().unwrap();
    let client_two = HttpClient::new(user_two, server_api_url.clone())?;
    println!(
        "new hub:\n  id: {}\n  name: {}\n  channel: {}",
        hub.id, hub.name, channel_id
    );
    let mut ws_client_one = WebsocketClient::new(user_one, "ws://localhost:8080/api").await?;

    ws_client_one
        .send_message(ClientMessage::SubscribeHub { hub_id })
        .await?;
    ws_client_one
        .send_message(ClientMessage::SubscribeChannel { hub_id, channel_id })
        .await?;

    let event_loop = tokio::spawn(ws_client_one.event_loop::<_, ()>(|_client, message| {
        match message {
            wicrs_server::websocket::ServerMessage::ChatMessage {
                sender_id,
                hub_id: _,
                channel_id: _,
                message_id: _,
                message,
            } => println!("{} sent '{}'", sender_id, message),
            wicrs_server::websocket::ServerMessage::HubUpdated {
                hub_id,
                update_type,
            } => match update_type {
                wicrs_server::server::HubUpdateType::UserJoined(user_id) => {
                    println!("{} joined {}", user_id, hub_id)
                }
                wicrs_server::server::HubUpdateType::UserLeft(user_id) => {
                    println!("{} left {}", user_id, hub_id);
                    return Some(());
                }
                _ => (),
            },
            _ => (),
        };
        None
    }));

    client_two.hub_join(hub_id).await?;

    client_two
        .message_send(hub_id, channel_id, "Hello world!".to_string())
        .await?;

    client_two.hub_leave(hub_id).await?;

    let _ = event_loop.await;

    Ok(())
}
