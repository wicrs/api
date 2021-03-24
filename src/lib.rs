use config::{ClientBuilderConfid, ClientConfig};
use reqwest::{header::HeaderMap, StatusCode};
pub use result::{Error, Result};
use wicrs_server::{ApiError, ID, auth::{IDToken, Service}, get_system_millis, user::User};

pub mod config;
pub mod result;

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

macro_rules! json_req {
    ($path:expr, $type:ident, $self:ident) => {
        if let Ok(response) = $self.client.get(&format!("{}/{}", $self.server_url, $path)).send().await {
            if let Ok(body) = response.text().await {
                if let Ok(error) = serde_json::from_str::<ApiError>(&body) {
                    return Err(error.into());
                }
                return serde_json::from_str::<$type>(&body).map_err(|_| crate::result::Error::UnexpectedResponse)
            } else {
                Err(crate::result::Error::Connection)
            }
        } else {
            Err(crate::result::Error::Connection)
        }
    };
}

macro_rules! empty_req {
    ($path:expr, $self:ident) => {
        if let Ok(response) = $self.client.get(&format!("{}/{}", $self.server_url, $path)).send().await {
            if response.status().is_success() {
                return Ok(());
            }
            if let Ok(body) = response.text().await {
                if let Ok(error) = serde_json::from_str::<ApiError>(&body) {
                    Err(error.into())
                } else {
                    Err(crate::result::Error::Connection)
                }
            } else {
                Err(crate::result::Error::Connection)
            }
        } else {
            Err(crate::result::Error::Connection)
        }
    };
}

impl Client {
    pub async fn get_user(&self) -> Result<User> {
        json_req!("user", User, self)
    }

    pub async fn invalidate_tokens(&self) -> Result<()> {
        empty_req!("invalidate_tokens", self)
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

    pub fn from_config(config: ClientBuilderConfid) -> Self {
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

    pub async fn finish_login(&mut self, id_token: IDToken, expiry: u128) {
        self.token_expiry = Some(expiry);
        self.user_id = Some(id_token.id);
        self.auth_token = Some(id_token.token);
    }

    pub async fn build(self) -> Result<Client> {
        Client::from_config(ClientConfig {
            user_id: self
                .user_id
                .map_or_else(|| Err(Error::LoginNotComplete), |id| Ok(id))?,
            auth_token: self
                .auth_token
                .map_or_else(|| Err(Error::LoginNotComplete), |token| Ok(token))?,
            token_expires: self
                .token_expiry
                .map_or_else(|| Err(Error::LoginNotComplete), |time| Ok(time))?,
            server_url: self.server_url,
        })
    }
}
