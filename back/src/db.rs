use mongodb::{self, ThreadedClient};
use mongodb::db::ThreadedDatabase;
use mongodb::coll::options::{IndexModel, IndexOptions};

use model::Gender;

pub const NAME: &'static str = "lsys-pairwise";
pub const COLLECTION_SAMPLE: &'static str = "sample";
pub const COLLECTION_USER: &'static str = "user";
pub const COLLECTION_WEIGHT: &'static str = "weight";

#[derive(Serialize, Deserialize)]
pub struct Sample {
    pub name: String,
}

#[derive(Serialize)]
pub struct User {
    pub age: i32,
    pub gender: Gender,
    pub token: String,
}

pub fn init(db_client: &mongodb::Client) -> mongodb::Result<()> {
    let db = db_client.db(NAME);

    db.collection(COLLECTION_SAMPLE).create_index(
        doc! { "name": 1 },
        Some(IndexOptions {
            unique: Some(true),
            ..Default::default()
        }),
    )?;

    db.collection(COLLECTION_USER).create_index(
        doc! { "token": 1 },
        Some(IndexOptions {
            unique: Some(true),
            ..Default::default()
        }),
    )?;

    db.collection(COLLECTION_WEIGHT).create_indexes(vec![
        IndexModel::new(
            doc! { "token": 1, "metric": 1, "a": 1, "b": 1 },
            Some(IndexOptions {
                unique: Some(true),
                ..Default::default()
            }),
        ),
        IndexModel::new(
            doc! { "metric": 1, "a": 1, "b": 1 },
            Some(IndexOptions {
                unique: Some(false),
                ..Default::default()
            }),
        ),
        IndexModel::new(
            doc! { "a": 1, "b": 1 },
            Some(IndexOptions {
                unique: Some(false),
                ..Default::default()
            }),
        ),
    ])?;

    Ok(())
}
