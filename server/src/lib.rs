pub mod app_state;
pub mod cli;
pub mod db;
pub mod item;
pub mod routes;
pub mod ui;
pub mod user_session;

use crate::app_state::{AppConfig, AppState};
use crate::cli::CliArgs;

use axum::{
    extract::{MatchedPath, Request},
    middleware::{self, Next},
    response::IntoResponse,
    routing, Router,
};
use axum_extra::routing::RouterExt;
use metrics_exporter_prometheus::{Matcher, PrometheusBuilder, PrometheusHandle};
use openidconnect::{
    core::{CoreClient, CoreProviderMetadata},
    ClientId, ClientSecret, IssuerUrl, RedirectUrl,
};
use sqlx::SqlitePool;
use std::future;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::time::Instant;

static INIT: std::sync::Once = std::sync::Once::new();

pub fn init_tracing() {
    INIT.call_once(|| {
        tracing_subscriber::fmt().init();
    });
}

/// Runs the server given the cli arguments parameters.
pub async fn run(
    CliArgs {
        listen_address,
        metrics_listen_address,
        host_url,
        database_url,
        openid_provider_url,
        openid_client_id,
        openid_client_secret,
    }: CliArgs,
) -> anyhow::Result<()> {
    init_tracing();

    let http_client = reqwest::ClientBuilder::new()
        // Following redirects opens the client up to SSRF vulnerabilities.
        .redirect(reqwest::redirect::Policy::none())
        .build()?;

    let provider_metadata = CoreProviderMetadata::discover_async(
        IssuerUrl::from_url(openid_provider_url),
        &http_client,
    )
    .await?;

    let redirect_url = host_url.join(&routes::OAuthCallbackPath.to_string())?;
    let client = CoreClient::from_provider_metadata(
        provider_metadata,
        ClientId::new(openid_client_id),
        Some(ClientSecret::new(openid_client_secret)),
    )
    // Set the URL the user will be redirected to after the authorization process.
    .set_redirect_uri(RedirectUrl::from_url(redirect_url));

    let app_state = Arc::new(AppConfig {
        db_pool: SqlitePool::connect(&database_url)
            .await
            .map_err(Into::<anyhow::Error>::into)?,
        http_client,
        openid_client: client,
        host_url,
    });

    let app = app(app_state);

    let tcp_listener = TcpListener::bind(listen_address).await?;
    tracing::info!("Server is listening on '{listen_address}'");

    match metrics_listen_address {
        None => axum::serve(tcp_listener, app).await.map_err(Into::into),
        Some(metrics_listen_address) => {
            let metrics_app = metrics_app();
            let metrics_tcp_listener =
                tokio::net::TcpListener::bind(metrics_listen_address).await?;
            tracing::info!("Metrics server is listening on {metrics_listen_address}");

            // Note that this does not spawn two top-level tasks, thus this will run
            // concurrently. To make this parrallel, tokio::task::spawn and then join.
            tokio::try_join!(
                axum::serve(tcp_listener, app),
                axum::serve(metrics_tcp_listener, metrics_app)
            )
            .map(|_| ())
            .map_err(|err| {
                tracing::error!("{err}");
                err.into()
            })
        }
    }
}

pub fn app(app_state: AppState) -> Router {
    Router::new()
        .typed_get(routes::get_items_handler)
        .typed_post(routes::new_item_handler)
        .typed_get(routes::get_status_handler)
        .typed_get(routes::get_login_handler)
        .typed_get(routes::get_logout_handler)
        .typed_get(routes::oauth_callback_handler)
        .route_layer(middleware::from_fn(track_metrics))
        .with_state(app_state)
}

pub fn metrics_app() -> Router {
    // Cannot be celled in future::ready`
    let metrics_recorder = install_metrics_recorder();
    Router::new().route(
        "/metrics",
        routing::get(move || future::ready(metrics_recorder.render())),
    )
}

pub fn install_metrics_recorder() -> PrometheusHandle {
    const EXPONENTIAL_SECONDS: &[f64] = &[
        0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0,
    ];
    PrometheusBuilder::new()
        .set_buckets_for_metric(
            Matcher::Full("http_requests_duration_seconds".to_string()),
            EXPONENTIAL_SECONDS,
        )
        .unwrap()
        .install_recorder()
        .unwrap()
}

pub async fn track_metrics(req: Request, next: Next) -> impl IntoResponse {
    let start = Instant::now();
    let path = match req.extensions().get::<MatchedPath>() {
        None => req.uri().path().to_owned(),
        Some(matched_path) => matched_path.as_str().to_owned(),
    };
    let method = req.method().clone();
    let response = next.run(req).await;

    let latency = start.elapsed().as_secs_f64();
    let status = response.status().as_u16().to_string();

    let labels = [
        ("method", method.to_string()),
        ("path", path),
        ("status", status),
    ];

    metrics::counter!("http_requests_total", &labels).increment(1);
    metrics::histogram!("http_requests_duration_seconds", &labels).record(latency);

    response
}
