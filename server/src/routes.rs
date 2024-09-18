use askama::Template;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{AppendHeaders, Html, IntoResponse, Response};
use axum_extra::routing::TypedPath;
use tracing::instrument;

use crate::app_state::AppState;
use crate::db;
use crate::item::ItemsTemplate;

#[derive(TypedPath, Debug, Clone, Copy)]
#[typed_path("/api/v1/items")]
pub struct GetItemsPath;

#[instrument(level = "trace", ret)]
pub async fn get_items_handler(_: GetItemsPath, state: State<AppState>) -> Html<String> {
    let items = db::get_items(&state.db_pool)
        .await
        .map_err(|err| {
            tracing::error!("{err}");
            err
        })
        .unwrap_or_default();
    let template = ItemsTemplate { items: &items };
    Html(template.render().unwrap())
}

#[derive(TypedPath, Debug, Clone, Copy)]
#[typed_path("/api/v1/items/new")]
pub struct NewItemPath;

#[instrument(level = "trace", ret)]
pub async fn new_item_handler(_: NewItemPath, state: State<AppState>) -> Response {
    match db::new_item(&state.db_pool, "test", 100).await {
        Ok(_) => AppendHeaders([("HX-Trigger", "newItem")]).into_response(),
        Err(err) => {
            tracing::error!("Failed to create new item:: {err}");
            (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()).into_response()
        }
    }
}
