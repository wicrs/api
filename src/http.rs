use chrono::{DateTime, Utc};
use reqwest::{
    header::{HeaderMap, HeaderValue},
    Body, Client, ClientBuilder, Method, Url,
};
use serde::{de::DeserializeOwned, Serialize};
use std::fmt::Display;
use wicrs_server::{
    channel::{Channel, Message},
    httpapi::Response,
    hub::HubMember,
    permission::{ChannelPermission, HubPermission, PermissionSetting},
};
use wicrs_server::{
    error::Result,
    httpapi::{
        handlers::message::{AfterQuery, TimePeriodQuery},
        routes::SetPermission,
    },
    hub::Hub,
    ID,
};

pub use wicrs_server::httpapi::handlers::channel::Update as ChannelUpdate;
pub use wicrs_server::httpapi::handlers::hub::Update as HubUpdate;
pub use wicrs_server::httpapi::handlers::member::Status as MemberStatus;

pub struct HttpClient {
    pub server_api_url: String,
    pub user_id: ID,
    client: Client,
}

impl HttpClient {
    pub fn new(user_id: ID, server_api_url: String) -> Result<Self> {
        let auth_string = user_id.to_string();
        let mut headers = HeaderMap::new();
        headers.insert(
            "authorization",
            HeaderValue::from_str(&auth_string).unwrap(),
        );
        let client = ClientBuilder::new()
            .default_headers(headers)
            .user_agent("WICRS Rust API")
            .build()?;
        Ok(Self {
            server_api_url,
            user_id,
            client,
        })
    }

    pub async fn request<S, R>(&self, method: Method, url: S) -> Result<R>
    where
        S: Display,
        R: DeserializeOwned,
    {
        let request = self
            .client
            .request(
                method,
                Url::parse(&format!("{}{}", self.server_api_url, url))?,
            )
            .build()?;
        let response = self
            .client
            .execute(request)
            .await?
            .json::<Response<R>>()
            .await?;
        match response {
            Response::Success(result) => Ok(result),
            Response::Error(error) => Err(error.into()),
        }
    }

    pub async fn request_norec<S>(&self, method: Method, url: S) -> Result
    where
        S: Display,
    {
        self.request::<_, String>(method, url).await?;
        Ok(())
    }

    pub async fn send<S, D, R>(&self, method: Method, url: S, data: D) -> Result<R>
    where
        S: Display,
        D: Into<Body>,
        R: DeserializeOwned,
    {
        let request = self
            .client
            .request(
                method,
                Url::parse(&format!("{}{}", self.server_api_url, url))?,
            )
            .body(data)
            .header("content-type", HeaderValue::from_static("application/json"))
            .build()?;
        let response = self
            .client
            .execute(request)
            .await?
            .json::<Response<R>>()
            .await?;
        match response {
            Response::Success(result) => Ok(result),
            Response::Error(error) => Err(error.into()),
        }
    }

    pub async fn send_json<S, D, R>(&self, method: Method, url: S, data: D) -> Result<R>
    where
        S: Display,
        D: Serialize,
        R: DeserializeOwned,
    {
        self.send(method, url, serde_json::to_string(&data)?).await
    }

    pub async fn send_json_norec<S, D>(&self, method: Method, url: S, data: D) -> Result<()>
    where
        S: Display,
        D: Serialize,
    {
        self.send_json::<_, _, String>(method, url, data).await?;
        Ok(())
    }
}

impl HttpClient {
    pub async fn hub_create(&self, name: String) -> Result<ID> {
        self.send(Method::POST, "/hub", name).await
    }

    pub async fn hub_get(&self, hub: ID) -> Result<Hub> {
        self.request(Method::GET, format!("/hub/{}", hub)).await
    }

    pub async fn hub_update(
        &self,
        hub: ID,
        name: Option<String>,
        description: Option<String>,
        default_group: Option<ID>,
    ) -> Result<HubUpdate> {
        let update = HubUpdate {
            name,
            description,
            default_group,
        };
        self.send_json(Method::POST, format!("/hub/{}", hub), update)
            .await
    }

    pub async fn hub_delete(&self, hub: ID) -> Result<()> {
        self.request_norec(Method::DELETE, format!("/hub/{}", hub))
            .await
    }

    pub async fn hub_join(&self, hub: ID) -> Result<()> {
        self.request_norec(Method::POST, format!("/hub/{}/join", hub))
            .await
    }

    pub async fn hub_leave(&self, hub: ID) -> Result<()> {
        self.request_norec(Method::POST, format!("/hub/{}/leave", hub))
            .await
    }
}

impl HttpClient {
    pub async fn message_get(&self, hub: ID, channel: ID, message: ID) -> Result<Message> {
        self.request(
            Method::GET,
            format!("/message/{}/{}/{}", hub, channel, message),
        )
        .await
    }

    pub async fn message_get_after(
        &self,
        hub: ID,
        channel: ID,
        from: ID,
        max: usize,
    ) -> Result<Vec<Message>> {
        self.send_json(
            Method::GET,
            format!("/message/{}/{}/after", hub, channel),
            AfterQuery { from, max },
        )
        .await
    }

    pub async fn message_get_time_period(
        &self,
        hub: ID,
        channel: ID,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
        max: usize,
        new_to_old: bool,
    ) -> Result<Vec<Message>> {
        self.send_json(
            Method::GET,
            format!("/message/{}/{}/time_period", hub, channel),
            TimePeriodQuery {
                from,
                to,
                max,
                new_to_old,
            },
        )
        .await
    }

    pub async fn message_send(&self, hub: ID, channel: ID, message: String) -> Result<ID> {
        self.send(
            Method::POST,
            format!("/message/{}/{}", hub, channel),
            message,
        )
        .await
    }
}

impl HttpClient {
    pub async fn channel_get(&self, hub: ID, channel: ID) -> Result<Channel> {
        self.request(Method::GET, format!("/channel/{}/{}", hub, channel))
            .await
    }

    pub async fn channel_create(&self, hub: ID, name: String) -> Result<ID> {
        self.send(Method::POST, format!("/channel/{}", hub), name)
            .await
    }

    pub async fn channel_update(
        &self,
        hub: ID,
        channel: ID,
        update: ChannelUpdate,
    ) -> Result<ChannelUpdate> {
        self.send_json(Method::PUT, format!("/channel/{}/{}", hub, channel), update)
            .await
    }

    pub async fn channel_delete(&self, hub: ID, channel: ID) -> Result<()> {
        self.request_norec(Method::DELETE, format!("/channel/{}/{}", hub, channel))
            .await
    }
}

impl HttpClient {
    pub async fn member_status(&self, hub: ID, member: ID) -> Result<MemberStatus> {
        self.request(Method::GET, format!("/member/{}/{}/status", hub, member))
            .await
    }

    pub async fn member_get(&self, hub: ID, member: ID) -> Result<HubMember> {
        self.request(Method::GET, format!("/member/{}/{}", hub, member))
            .await
    }

    pub async fn member_kick(&self, hub: ID, member: ID) -> Result<()> {
        self.request_norec(Method::POST, format!("/member/{}/{}/kick", hub, member))
            .await
    }

    pub async fn member_ban(&self, hub: ID, member: ID) -> Result<()> {
        self.request_norec(Method::POST, format!("/member/{}/{}/ban", hub, member))
            .await
    }

    pub async fn member_unban(&self, hub: ID, member: ID) -> Result<()> {
        self.request_norec(Method::POST, format!("/member/{}/{}/unban", hub, member))
            .await
    }

    pub async fn member_mute(&self, hub: ID, member: ID) -> Result<()> {
        self.request_norec(Method::POST, format!("/member/{}/{}/mute", hub, member))
            .await
    }

    pub async fn member_unmute(&self, hub: ID, member: ID) -> Result<()> {
        self.request_norec(Method::POST, format!("/member/{}/{}/unmute", hub, member))
            .await
    }

    pub async fn member_get_hub_permission(
        &self,
        hub: ID,
        member: ID,
        permission: HubPermission,
    ) -> Result<PermissionSetting> {
        self.request(
            Method::GET,
            format!("/member/{}/{}/hub_permission/{}", hub, member, permission),
        )
        .await
    }

    pub async fn member_set_hub_permission(
        &self,
        hub: ID,
        member: ID,
        permission: HubPermission,
        setting: PermissionSetting,
    ) -> Result<()> {
        self.send_json_norec(
            Method::PUT,
            format!("/member/{}/{}/hub_permission/{}", hub, member, permission),
            SetPermission { setting },
        )
        .await
    }

    pub async fn member_get_channel_permission(
        &self,
        hub: ID,
        member: ID,
        channel: ID,
        permission: ChannelPermission,
    ) -> Result<PermissionSetting> {
        self.request(
            Method::GET,
            format!(
                "/member/{}/{}/channel_permission/{}/{}",
                hub, member, channel, permission
            ),
        )
        .await
    }

    pub async fn member_set_channel_permission(
        &self,
        hub: ID,
        member: ID,
        permission: ChannelPermission,
        setting: PermissionSetting,
    ) -> Result<()> {
        self.send_json_norec(
            Method::PUT,
            format!(
                "/member/{}/{}/channel_permission/{}",
                hub, member, permission
            ),
            SetPermission { setting },
        )
        .await
    }
}
