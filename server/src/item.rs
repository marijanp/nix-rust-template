use askama::Template;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Item {
    pub id: Uuid,
    pub name: String,
    pub price: u128,
}

#[derive(Template)]
#[template(path = "items.html")]
pub struct ItemsTemplate<'a> {
    pub items: &'a Vec<Item>,
}
