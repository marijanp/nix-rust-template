use askama::Template;
use axum::response::Html;
use axum::extract::State;
use axum_extra::routing::TypedPath;
use lazy_static::lazy_static;
use std::sync::Mutex;
use tracing::instrument;

use crate::item::{Item, ItemsTemplate};

lazy_static! {
    static ref ITEMS: Mutex<Vec<Item>> = Mutex::new(vec![Item {
        id: uuid::Uuid::new_v4(),
        name: "test".to_string(),
        price: 12345
    }]);
}
use crate::app_state::AppState;

#[derive(TypedPath, Debug, Clone, Copy)]
#[typed_path("/api/v1/items")]
pub struct GetItemsPath;

#[instrument(level = "trace", ret)]
    let items = ITEMS.lock().unwrap();
pub async fn get_items_handler(_: GetItemsPath, state: State<AppState>) -> Html<String> {
    let template = ItemsTemplate { items: &items };
    Html(template.render().unwrap())
}
