use openidconnect::{core::CoreClient, EndpointMaybeSet, EndpointNotSet, EndpointSet};
use reqwest::Client;
use sqlx::SqlitePool;
use std::sync::Arc;
use url::Url;

pub type AppState = Arc<AppConfig>;

#[derive(Debug)]
pub struct AppConfig {
    pub db_pool: SqlitePool,
    pub http_client: Client,
    pub openid_client: OpenIdClient,
    pub host_url: Url,
}

pub type OpenIdClient = CoreClient<
    EndpointSet,      // HasAuthUrl
    EndpointNotSet,   // HasDeviceAuthUrl
    EndpointNotSet,   // HasIntrospectionUrl
    EndpointNotSet,   // HasRevocationUrl
    EndpointMaybeSet, // HasTokenUrl
    EndpointMaybeSet, // HasUserInfoUrl
>;
