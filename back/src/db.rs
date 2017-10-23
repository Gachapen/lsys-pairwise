use mongodb::{self, ThreadedClient};
use mongodb::db::ThreadedDatabase;
use mongodb::coll::options::IndexOptions;

use model::Gender;

pub const NAME: &'static str = "lsys-pairwise";
pub const COLLECTION_SAMPLE: &'static str = "sample";
pub const COLLECTION_USER: &'static str = "user";

#[derive(Serialize)]
pub struct Sample {
    pub name: String,
}

#[derive(Serialize)]
pub struct User {
    pub age: i32,
    pub gender: Gender,
}

pub fn init(db_client: &mongodb::Client) -> mongodb::Result<()> {
    db_client
        .db(NAME)
        .collection(COLLECTION_SAMPLE)
        .create_index(
            doc! { "name": 1 },
            Some(IndexOptions {
                unique: Some(true),
                ..Default::default()
            }),
        )?;

    Ok(())
}
