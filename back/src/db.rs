use bson::oid::ObjectId;
use chrono::{NaiveDateTime, Utc};
use mongodb::{self, Client, ThreadedClient};
use mongodb::error::Error;
use mongodb::db::ThreadedDatabase;
use mongodb::coll::options::{FindOptions, IndexModel, IndexOptions};
use uuid::Uuid;

use serde_enum;
use model::{self, Gender, Metric, PostQuestionnaire, PreQuestionnaire};
use cfg;

pub const NAME: &str = "lsys-pairwise";
pub const COLLECTION_SAMPLE: &str = "sample";
pub const COLLECTION_USER: &str = "user";
pub const COLLECTION_WEIGHT: &str = "weight";

#[derive(Serialize, Deserialize)]
pub struct User {
    pub age: i32,
    pub gender: Gender,
    pub token: String,
    pub task: String,
    pub register_date: NaiveDateTime,
    pub pre_questionnaire: Option<PreQuestionnaire>,
    pub post_questionnaire: Option<PostQuestionnaire>,
}

impl From<model::User> for User {
    fn from(user: model::User) -> User {
        User {
            age: i32::from(user.age),
            gender: user.gender,
            token: format!("{}", Uuid::new_v4().simple()),
            task: user.task,
            register_date: Utc::now().naive_utc(),
            pre_questionnaire: user.pre_questionnaire,
            post_questionnaire: None,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Weighting {
    pub token: String,
    pub metric: Metric,
    pub a: ObjectId,
    pub b: ObjectId,
    pub weight: f32,
    pub time: NaiveDateTime,
}

impl From<model::Weighting> for Weighting {
    fn from(weighting: model::Weighting) -> Weighting {
        Weighting {
            token: weighting.token,
            metric: weighting.metric,
            a: ObjectId::with_string(&weighting.a).unwrap(),
            b: ObjectId::with_string(&weighting.b).unwrap(),
            weight: weighting.weight,
            time: Utc::now().naive_utc(),
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
    let client = Client::connect(&db_cfg.host, 27_017)
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
    let collection = db_client.db(NAME).collection(COLLECTION_WEIGHT);

    let doc_a = collection.find_one(
        Some(doc! {
            "token": user_token,
            // FIXME: This is a hack to support only the pleasing metric.
            "metric": serde_enum::to_string(&Metric::Pleasing).unwrap(),
            "a": id_a.clone(),
            "b": id_b.clone(),
        }),
        Some(FindOptions {
            projection: Some(doc! {
                "_id": 0,
            }),
            ..Default::default()
        }),
    )?;

    if doc_a.is_some() {
        return Ok(false);
    }

    let doc_b = collection.find_one(
        Some(doc! {
            "token": user_token,
            // FIXME: This is a hack to support only the pleasing metric.
            "metric": serde_enum::to_string(&Metric::Pleasing).unwrap(),
            "a": id_b.clone(),
            "b": id_a.clone(),
        }),
        Some(FindOptions {
            projection: Some(doc! {
                "_id": 0,
            }),
            ..Default::default()
        }),
    )?;

    Ok(doc_b.is_none())
}
