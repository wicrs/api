use reqwest::{
    header::{HeaderMap, HeaderValue},
    Client, ClientBuilder, Method, Url,
};
use wicrs_server::{error::{Result}, hub::Hub, ID};
pub use wicrs_server::httpapi::handlers::hub::Update as HubUpdate;

pub struct HttpClient {
    pub server_url: String,
    pub user_id: ID,
    client: Client,
}

impl HttpClient {
    pub fn new(user_id: ID, server_url: String) -> Result<Self> {
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
            server_url,
            user_id,
            client,
        })
    }

    pub fn request<T, R>(&self, url: String, data: T) -> Result<R> {
        
    }
}

impl HttpClient {
    pub async fn hub_create(&self, name: String) -> Result<ID> {
        let request = self
            .client
            .request(
                Method::POST,
                Url::parse(&format!("{}/hub", self.server_url))?,
            )
            .body(name)
            .build()?;
        let text = dbg!(self.client.execute(request).await?.text().await?);
        Ok(ID::parse_str(&text)?)
    }

    pub async fn hub_get(&self, id: ID) -> Result<Hub> {
        let request = self
            .client
            .request(
                Method::GET,
                Url::parse(&format!("{}/hub/{}", self.server_url, id))?,
            )
            .build()?;
        Ok(self.client.execute(request).await?.json::<Hub>().await?)
    }

    pub async fn hub_update(
        &self,
        id: ID,
        name: Option<String>,
        description: Option<String>,
        default_group: Option<ID>,
    ) -> Result<HubUpdate> {
        let update = HubUpdate {
            name,
            description,
            default_group,
        };
        let request = self
            .client
            .request(
                Method::POST,
                Url::parse(&format!("{}/hub/{}", self.server_url, id))?,
            )
            .json(&update)
            .build()?;
        Ok(self
            .client
            .execute(request)
            .await?
            .json::<HubUpdate>()
            .await?)
    }

    pub async fn hub_delete(&self, id: ID) -> Result<()> {

    }
}
