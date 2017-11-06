use bson::oid::ObjectId;
use mongodb::{self, Client, ThreadedClient};
use mongodb::error::Error;
use mongodb::db::ThreadedDatabase;
use mongodb::coll::options::{FindOptions, IndexModel, IndexOptions};

use serde_enum;
use model::{self, Gender, Metric};
use cfg;

pub const NAME: &'static str = "lsys-pairwise";
pub const COLLECTION_SAMPLE: &'static str = "sample";
pub const COLLECTION_USER: &'static str = "user";
pub const COLLECTION_WEIGHT: &'static str = "weight";

#[derive(Serialize, Deserialize)]
pub struct User {
    pub age: i32,
    pub gender: Gender,
    pub token: String,
    pub task: String,
}

#[derive(Serialize, Deserialize)]
pub struct Weighting {
    pub token: String,
    pub metric: Metric,
    pub a: ObjectId,
    pub b: ObjectId,
    pub weight: f32,
}

impl From<model::Weighting> for Weighting {
    fn from(weighting: model::Weighting) -> Weighting {
        Weighting {
            token: weighting.token,
            metric: weighting.metric,
            a: ObjectId::with_string(&weighting.a).unwrap(),
            b: ObjectId::with_string(&weighting.b).unwrap(),
            weight: weighting.weight,
        }
    }
}

pub fn init(db_client: &mongodb::Client) -> mongodb::Result<()> {
    let db = db_client.db(NAME);

    db.collection(COLLECTION_SAMPLE).create_indexes(vec![
        IndexModel::new(
            doc! { "task": 1, "name": 1 },
            Some(IndexOptions {
                unique: Some(true),
                ..Default::default()
            }),
        ),
        IndexModel::new(
            doc! { "task": 1 },
            Some(IndexOptions {
                unique: Some(false),
                ..Default::default()
            }),
        ),
    ])?;

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
            doc! { "token": 1, "metric": 1 },
            Some(IndexOptions {
                unique: Some(false),
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

pub fn connect(db_cfg: &cfg::Db) -> Client {
    let client = Client::connect(&db_cfg.host, 27017)
        .expect(&format!("Failed to connect to DB at {}", db_cfg.host));
    println!("Connected to MongoDB at {}", db_cfg.host);

    client
}

pub fn missing_measurement(
    user_token: &str,
    id_a: &ObjectId,
    id_b: &ObjectId,
    db_client: &Client,
) -> Result<bool, Error> {
    let doc = db_client.db(NAME).collection(COLLECTION_WEIGHT).find_one(
        Some(doc! {
            "token": user_token,
            "metric": {
                "$in": [
                    serde_enum::to_string(&Metric::Realistic).unwrap(),
                    serde_enum::to_string(&Metric::Pleasing).unwrap(),
                ],
            },
            "a": {
                "$in": [id_a.clone(), id_b.clone()],
            },
            "b": {
                "$in": [id_b.clone(), id_a.clone()],
            },
        }),
        Some(FindOptions {
            projection: Some(doc! {
                "_id": 0,
            }),
            ..Default::default()
        }),
    )?;

    Ok(doc.is_none())
}
