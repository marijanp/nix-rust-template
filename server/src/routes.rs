use askama::Template;
use axum::response::Html;
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

#[derive(TypedPath, Debug, Clone, Copy)]
#[typed_path("/api/v1/items")]
pub struct GetItemsPath;

#[instrument(level = "trace", ret)]
pub async fn get_items_handler(_: GetItemsPath) -> Html<String> {
    let items = ITEMS.lock().unwrap();
    let template = ItemsTemplate { items: &items };
    Html(template.render().unwrap())
}
