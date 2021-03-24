#![feature(io_read_to_string)]

use std::io::BufRead;

use wicrs_api::*;
use wicrs_server::auth::{IDToken, Service};

#[tokio::main]
#[allow(unused_must_use)]
async fn main() -> Result<()> {
    let mut builder = ClientBuilder::new("http://localhost:8080/v2", Service::GitHub);
    builder.start_login().await.expect("Failed to start login process.");
    println!("Go to this URL and paste the result below: {}", builder.start_login().await.expect("Failed to start login process."));
    let stdin = std::io::stdin();
    let line = stdin.lock()
        .lines()
        .next()
        .expect("there was no next line")
        .expect("the line could not be read");
    builder.finish_login(serde_json::from_str::<IDToken>(&line).expect("Invalid input."), u128::MAX);
    println!("Good IDToken, getting client.");
    let client = builder.build().await.expect("Failed to make a client.");
    dbg!(client.get_user().await);
    dbg!(client.get_user_by_id(&client.user_id).await);
    dbg!(client.change_username("oooooo").await);
    let hub = client.create_hub("test").await?;
    dbg!(client.get_user().await);
    dbg!(client.get_hub(&hub).await);
    Ok(())
}
