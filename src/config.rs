use std::convert::TryFrom;

use serde::{Deserialize, Serialize};
use wicrs_server::{auth::Service, ID};

use crate::{ClientBuilder, Error};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ClientConfig {
    pub user_id: ID,
    pub auth_token: String,
    pub token_expires: u128,
    pub server_url: String,
}

impl TryFrom<ClientBuilder> for ClientConfig {
    type Error = Error;

    fn try_from(value: ClientBuilder) -> Result<Self, Self::Error> {
        Ok(Self {
            user_id: value
                .user_id
                .map_or_else(|| Err(Error::LoginNotComplete), |id| Ok(id))?,
            auth_token: value
                .auth_token
                .map_or_else(|| Err(Error::LoginNotComplete), |token| Ok(token))?,
            token_expires: value
                .token_expiry
                .map_or_else(|| Err(Error::LoginNotComplete), |time| Ok(time))?,
            server_url: value.server_url,
        })
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ClientBuilderConfig {
    pub server_url: String,
    pub auth_service: Service,
}

impl From<ClientBuilder> for ClientBuilderConfig {
    fn from(builder: ClientBuilder) -> Self {
        Self {
            server_url: builder.server_url,
            auth_service: builder.auth_service,
        }
    }
}
