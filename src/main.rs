use wicrs_api::*;
use wicrs_server::auth::Service;

#[tokio::main]
async fn main() {
    let builder = ClientBuilder::new("http://localhost:8080/v2", Service::GitHub);
    dbg!(builder.start_login().await);
}
