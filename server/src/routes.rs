use axum::http::StatusCode;
use axum_extra::routing::TypedPath;
use tracing::instrument;

#[derive(TypedPath, Debug, Clone, Copy)]
#[typed_path("/api/v1/items")]
pub struct GetItemsPath;

#[instrument(level = "trace", ret)]
pub async fn get_items_handler(_: GetItemsPath) -> (StatusCode, &'static str) {
    (StatusCode::OK, "Hello World!")
}
