use reqwest::{StatusCode, Url};
use wicrs_common::{
    api_types::{ChannelCreateQuery, CreateAccountQuery, HubCreateQuery, MessageSendQuery},
    types::{Account, GenericUser, User},
    ID, NAME_ALLOWED_CHARS,
};

#[derive(Debug, PartialEq, Eq)]
pub enum ApiError {
    GetFailed,
    ServerError,
    EmptyResponse,
    InvalidAuthentication,
    BadName,
    NotFound,
    ParseFailed,
    NoPermisson,
}

pub type Result<T> = std::result::Result<T, ApiError>;

pub async fn get<T: serde::ser::Serialize>(url: Url, data: Option<T>) -> Result<String> {
    let request = reqwest::Client::new().get(url);
    let response;
    if let Some(data) = data {
        if let Ok(result) = request.json(&data).send().await {
            response = result;
        } else {
            return Err(ApiError::GetFailed);
        }
    } else {
        if let Ok(result) = request.send().await {
            response = result;
        } else {
            return Err(ApiError::GetFailed);
        }
    }
    let status = response.status();
    if status.is_server_error() {
        Err(ApiError::ServerError)
    } else {
        if status == StatusCode::NOT_FOUND {
            Err(ApiError::NotFound)
        } else {
            if let Ok(text) = response.text().await {
                match text.as_str() {
                    "Invalid authentication details." => Err(ApiError::InvalidAuthentication),
                    _ => {
                        if text
                            == format!(
                                "Username string can only contain the following characters: \"{}\"",
                                NAME_ALLOWED_CHARS
                            )
                        {
                            Err(ApiError::BadName)
                        } else {
                            Ok(text)
                        }
                    }
                }
            } else {
                if status.is_success() {
                    Ok(status.to_string())
                } else {
                    Err(ApiError::EmptyResponse)
                }
            }
        }
    }
}

pub struct Client {
    id: String,
    token: String,
    server: String,
}

impl Client {
    pub fn new(id: String, token: String, server: String) -> Self {
        Self { id, token, server }
    }

    pub fn auth(&self, endpoint: &str) -> Url {
        Url::parse(&format!(
            "{}/{}?user={}&token={}",
            self.server, endpoint, self.id, self.token
        ))
        .unwrap()
    }

    pub async fn get_own_user(&self) -> Result<User> {
        let response = get(self.auth("api/v1/user"), None::<()>).await?;
        if let Ok(user) = serde_json::from_str(&response) {
            Ok(user)
        } else {
            Err(ApiError::ParseFailed)
        }
    }

    pub async fn get_invalidate_tokens(&self) -> Result<()> {
        get(self.auth("api/v1/invalidate"), None::<()>).await?;
        Ok(())
    }

    pub async fn get_user(&self, id: String) -> Result<GenericUser> {
        let response = get(self.auth(&format!("api/v1/user/{}", id)), None::<()>).await?;
        if let Ok(user) = serde_json::from_str(&response) {
            Ok(user)
        } else {
            Err(ApiError::ParseFailed)
        }
    }

    pub async fn create_account(&self, name: String, bot: bool) -> Result<Account> {
        let response = get(
            self.auth("api/v1/user/addacount"),
            Some(CreateAccountQuery { name, is_bot: bot }),
        )
        .await?;
        if let Ok(account) = serde_json::from_str(&response) {
            Ok(account)
        } else {
            Err(ApiError::ParseFailed)
        }
    }

    pub async fn create_hub(&self, account: ID, name: String) -> Result<ID> {
        let response = get(
            self.auth("api/v1/hub/create"),
            Some(HubCreateQuery { name, account }),
        )
        .await?;
        if let Ok(hub) = response.parse() {
            Ok(hub)
        } else {
            Err(ApiError::ParseFailed)
        }
    }

    pub async fn create_channel(&self, account: ID, hub: ID, name: String) -> Result<ID> {
        let response = get(
            self.auth("api/v1/hub/create_channel"),
            Some(ChannelCreateQuery { name, account, hub }),
        )
        .await?;
        if let Ok(channel) = response.parse() {
            Ok(channel)
        } else {
            Err(ApiError::ParseFailed)
        }
    }

    pub async fn send_message(
        &self,
        account: ID,
        hub: ID,
        channel: ID,
        message: String,
    ) -> Result<ID> {
        let response = get(
            self.auth("api/v1/hub/send_message"),
            Some(MessageSendQuery {
                message,
                account,
                hub,
                channel,
            }),
        )
        .await?;
        if let Ok(message) = response.parse() {
            Ok(message)
        } else {
            Err(ApiError::ParseFailed)
        }
    }
}
