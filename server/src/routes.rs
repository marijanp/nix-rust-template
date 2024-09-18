use askama::Template;
use axum::response::Html;
use axum::extract::State;
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
