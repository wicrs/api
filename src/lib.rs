use config::{ClientBuilderConfig, ClientConfig};
use reqwest::{header::HeaderMap, StatusCode};
pub use result::{Error, Result};
use std::{convert::TryInto, str::FromStr};
use wicrs_server::{
    auth::{IDToken, Service},
    get_system_millis,
    hub::Hub,
    user::{GenericUser, User},
    ApiError, ID,
};

pub mod config;
pub mod result;
#[macro_use]
mod macros;

pub struct Client {
    pub server_url: String,
    pub user_id: ID,
    pub token_expires: u128,
    client: reqwest::Client,
}

pub struct ClientBuilder {
    server_url: String,
    user_id: Option<ID>,
    auth_token: Option<String>,
    token_expiry: Option<u128>,
    auth_service: Service,
}

impl Client {
    pub fn from_config(config: ClientConfig) -> Result<Self> {
        if get_system_millis() > config.token_expires {
            return Err(Error::TokenExpired);
        } else {
            let mut header_map = HeaderMap::new();
            let header_value: reqwest::header::HeaderValue =
                format!("{}:{}", &config.user_id, config.auth_token)
                    .parse()
                    .map_err(|_| Error::ReqwestClientBuild)?;
            header_map.insert(reqwest::header::AUTHORIZATION, header_value);
            let reqwest_client = reqwest::Client::builder()
                .redirect(reqwest::redirect::Policy::none())
                .default_headers(header_map)
                .build()
                .map_err(|_| Error::ReqwestClientBuild)?;
            return Ok(Self {
                server_url: config.server_url,
                user_id: config.user_id,
                token_expires: config.token_expires,
                client: reqwest_client,
            });
        }
    }
}

impl Client {
    pub async fn invalidate_tokens(&self) -> Result<()> {
        get!("invalidate_tokens", self)
    }

    pub async fn get_user(&self) -> Result<User> {
        get!("user", User, self)
    }

    pub async fn get_user_by_id(&self, id: &ID) -> Result<GenericUser> {
        get!(format!("user/{}", id), GenericUser, self)
    }

    pub async fn change_username<S: Into<String>>(&self, new_name: S) -> Result<String> {
        put!(
            format!("user/change_username/{}", new_name.into()),
            String,
            self
        )
    }

    pub async fn create_hub<S: Into<String>>(&self, name: S) -> Result<ID> {
        post!(format!("hub/create/{}", name.into()), ID, self)
    }

    pub async fn get_hub(&self, id: &ID) -> Result<Hub> {
        get!(format!("hub/{}", id), Hub, self)
    }
}

impl ClientBuilder {
    pub fn new<S: Into<String>>(server_url: S, auth_service: Service) -> Self {
        Self {
            server_url: server_url.into(),
            user_id: None,
            auth_token: None,
            token_expiry: None,
            auth_service,
        }
    }

    pub fn from_config(config: ClientBuilderConfig) -> Self {
        Self::new(config.server_url, config.auth_service)
    }

    pub async fn start_login(&self) -> Result<String> {
        let client = reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .map_err(|_| Error::ReqwestClientBuild)?;
        let response = client
            .get(&format!("{}/login/{}", self.server_url, self.auth_service))
            .send()
            .await
            .map_err(|_| Error::Connection)?;
        if response.status() == StatusCode::FOUND {
            if let Some(header) = response.headers().get(reqwest::header::LOCATION) {
                return Ok(header
                    .to_str()
                    .map_err(|_| Error::UnexpectedResponse)?
                    .to_string());
            }
        }
        Err(Error::UnexpectedResponse)
    }

    pub fn finish_login(&mut self, id_token: IDToken, expiry: u128) {
        self.token_expiry = Some(expiry);
        self.user_id = Some(id_token.id);
        self.auth_token = Some(id_token.token);
    }

    pub async fn build(self) -> Result<Client> {
        Client::from_config(self.try_into()?)
    }
}
