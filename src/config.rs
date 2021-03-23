use wicrs_server::{ID, auth::Service};
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ClientConfig {
    pub user_id: ID,
    pub auth_token: String,
    pub server_url: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ClientBuilderConfid {
    pub server_url: String,
    pub auth_service: Service,
}
