pub mod cli;
pub mod routes;

use crate::cli::CliArgs;

use axum::Router;
use axum_extra::routing::RouterExt;
use tokio::net::TcpListener;

/// Runs the server given the cli arguments parameters.
pub async fn run(CliArgs { listen_address }: CliArgs) -> std::io::Result<()> {
    tracing_subscriber::fmt::init();

    let app = app();
    let tcp_listener = TcpListener::bind(listen_address)
        .await
        .expect("Failed to bind to the listen address '{listen_address}'");

    tracing::info!("Server is listening on '{listen_address}'");
    axum::serve(tcp_listener, app).await
}

pub fn app() -> Router {
    Router::new().typed_get(routes::get_items_handler)
}
