extern crate wicrs_api;

use wicrs_api::{http::HttpClient, error::Result};

#[tokio::main]
pub async fn main() -> Result<()> {
    let client = HttpClient::new(
        wicrs_server::new_id(),
        "http://localhost:8080/api".to_string(),
    )?;
    let hub_id = client.hub_create("test".to_string()).await?;
    let hub = client.hub_get(hub_id).await?;
    dbg!(hub);
    Ok(())
}
