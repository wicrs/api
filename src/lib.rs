use config::{ClientBuilderConfig, ClientConfig};
use reqwest::{header::HeaderMap, StatusCode};
pub use result::{Error, Result};
use std::{convert::TryInto, fmt::Display, str::FromStr};
use wicrs_server::{
    auth::{IDToken, Service},
    channel::{Channel, Message},
    get_system_millis,
    hub::{Hub, HubMember},
    permission::{ChannelPermission, HubPermission, PermissionSetting},
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

    pub async fn get_user_by_id(&self, user_id: &ID) -> Result<GenericUser> {
        get!(format!("user/{}", user_id), GenericUser, self)
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

    pub async fn get_hub(&self, hub_id: &ID) -> Result<Hub> {
        get!(format!("hub/{}", hub_id), Hub, self)
    }

    pub async fn delete_hub(&self, hub_id: &ID) -> Result<()> {
        req!(format!("hub/{}", hub_id), delete, self)
    }

    pub async fn rename_hub<S: Into<String>>(&self, hub_id: &ID, new_name: S) -> Result<String> {
        put!(
            format!("hub/rename/{}/{}", hub_id, new_name.into()),
            String,
            self
        )
    }

    pub async fn is_banned_from_hub(&self, hub_id: &ID, user_id: &ID) -> Result<bool> {
        get!(
            format!("member/{}/{}/is_banned", hub_id, user_id),
            bool,
            self
        )
    }

    pub async fn hub_member_is_muted(&self, hub_id: &ID, user_id: &ID) -> Result<bool> {
        get!(
            format!("member/{}/{}/is_muted", hub_id, user_id),
            bool,
            self
        )
    }

    pub async fn get_hub_member(&self, hub_id: &ID, user_id: &ID) -> Result<HubMember> {
        get!(format!("member/{}/{}", hub_id, user_id), HubMember, self)
    }

    pub async fn join_hub(&self, hub_id: &ID) -> Result<()> {
        post!(format!("hub/join/{}", hub_id), self)
    }

    pub async fn leave_hub(&self, hub_id: &ID) -> Result<()> {
        post!(format!("hub/leave/{}", hub_id), self)
    }

    member_fns! {
        (kick_user, kick), (ban_user, ban), (unban_user, unban), (mute_user, mute), (unmute_user, unmute)
    }

    pub async fn change_nickname<S: Into<String>>(
        &self,
        hub_id: &ID,
        new_name: S,
    ) -> Result<String> {
        put!(
            format!("member/change_nickname/{}/{}", hub_id, new_name.into()),
            String,
            self
        )
    }

    pub async fn create_channel<S: Into<String>>(&self, hub_id: &ID, name: S) -> Result<ID> {
        put!(
            format!("channel/create/{}/{}", hub_id, name.into()),
            ID,
            self
        )
    }

    pub async fn get_channel(&self, hub_id: &ID, channel_id: &ID) -> Result<Channel> {
        get!(format!("channel/{}/{}", hub_id, channel_id), Channel, self)
    }

    pub async fn delete_channel(&self, hub_id: &ID, channel_id: &ID) -> Result<()> {
        req!(format!("channel/{}/{}", hub_id, channel_id), delete, self)
    }

    pub async fn send_message<D: Display>(
        &self,
        hub_id: &ID,
        channel_id: &ID,
        message: D,
    ) -> Result<ID> {
        post!(
            format!("message/send/{}/{}/{}", hub_id, channel_id, message),
            ID,
            self
        )
    }

    pub async fn get_message(
        &self,
        hub_id: &ID,
        channel_id: &ID,
        message_id: &ID,
    ) -> Result<Message> {
        get!(
            format!("message/{}/{}/{}", hub_id, channel_id, message_id),
            Message,
            self
        )
    }

    pub async fn get_messages(
        &self,
        hub_id: &ID,
        channel_id: &ID,
        from: Option<u128>,
        to: Option<u128>,
        invert: Option<bool>,
        max: Option<u128>,
    ) -> Result<Vec<Message>> {
        let mut query_str = String::new();
        if let Some(from) = from {
            query_str = format!("?from={}", from);
        }
        if let Some(to) = to {
            query_str = format!("{}&to={}", query_str, to);
        }
        if let Some(invert) = invert {
            query_str = format!("{}&invert={}", query_str, invert);
        }
        if let Some(max) = max {
            query_str = format!("{}&max={}", query_str, max);
        }
        type Messages = Vec<Message>;
        get!(
            format!("message/{}/{}{}", hub_id, channel_id, query_str),
            Messages,
            self
        )
    }

    pub async fn set_user_hub_permission(
        &self,
        hub_id: &ID,
        user_id: &ID,
        permission: HubPermission,
        setting: &PermissionSetting,
    ) -> Result<()> {
        post!(
            format!(
                "member/set_hub_permission/{}/{}/{}?setting={}",
                hub_id,
                user_id,
                permission,
                serde_json::to_string(setting).map_err(|_| Error::SerializeFailed)?
            ),
            self
        )
    }

    pub async fn set_user_channel_permission(
        &self,
        hub_id: &ID,
        channel_id: &ID,
        user_id: &ID,
        permission: ChannelPermission,
        setting: &PermissionSetting,
    ) -> Result<()> {
        post!(
            format!(
                "member/set_hub_permission/{}/{}/{}/{}?setting={}",
                hub_id,
                channel_id,
                user_id,
                permission,
                serde_json::to_string(setting).map_err(|_| Error::SerializeFailed)?
            ),
            self
        )
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
