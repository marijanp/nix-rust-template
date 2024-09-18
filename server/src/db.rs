use crate::item::Item;
use sqlx::{query_as, FromRow, SqlitePool};
use uuid::Uuid;

#[derive(FromRow)]
struct DbItem {
    id: String,
    name: String,
    price: String,
}

impl From<DbItem> for Item {
    fn from(db_item: DbItem) -> Item {
        Item {
            id: Uuid::parse_str(&db_item.id).expect(""),
            name: db_item.name,
            price: db_item.price.parse::<u128>().expect(""),
        }
    }
}

pub async fn new_item(pool: &SqlitePool, name: &str, price: u128) -> Result<Item, sqlx::Error> {
    let id = Uuid::new_v4().to_string();
    let price = price.to_string();
    query_as!(
        DbItem,
        "insert into items (id, name, price) values ($1, $2, $3) returning *",
        id,
        name,
        price
    )
    .fetch_one(pool)
    .await
    .map(Into::into)
}

/// Returns all known items
pub async fn get_items(pool: &SqlitePool) -> Result<Vec<Item>, sqlx::Error> {
    query_as!(DbItem, "select * from items")
        .fetch_all(pool)
        .await
        .map(|items| items.into_iter().map(Into::into).collect())
}
