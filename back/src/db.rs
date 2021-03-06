use bson::oid::ObjectId;
use chrono::{NaiveDateTime, Utc};
use mongodb::{self, Client, ThreadedClient};
use mongodb::db::ThreadedDatabase;
use mongodb::coll::options::{IndexModel, IndexOptions};
use uuid::Uuid;

use model::{self, Browser, Education, Gender, Metric, Occupation, PostQuestionnaire,
            PreQuestionnaire};
use cfg;

pub const NAME: &str = "lsys-pairwise";
pub const COLLECTION_SAMPLE: &str = "sample";
pub const COLLECTION_USER: &str = "user";
pub const COLLECTION_WEIGHT: &str = "weight";

/// A user representation in the database
#[derive(Serialize, Deserialize)]
pub struct User {
    pub age: i32,
    pub gender: Gender,
    pub education: Education,
    pub occupation: Occupation,
    /// Private token used to identify user in API
    pub token: String,
    /// Public token used to identify user without access to data
    pub public: String,
    /// Public token of user that shared the URL to this user, if any
    pub from: Option<String>,
    /// Source of the participation (such as 'url' or 'observation')
    pub source: String,
    /// Task assigned to user
    pub task: String,
    pub register_date: NaiveDateTime,
    pub pre_questionnaire: Option<PreQuestionnaire>,
    pub post_questionnaire: Option<PostQuestionnaire>,
    pub browser: Option<Browser>,
}

impl From<model::User> for User {
    fn from(user: model::User) -> User {
        User {
            age: i32::from(user.age),
            gender: user.gender,
            education: user.education,
            occupation: user.occupation,
            token: format!("{}", Uuid::new_v4().simple()),
            public: format!("{}", Uuid::new_v4().simple()),
            from: user.from,
            source: user.source,
            task: user.task,
            register_date: Utc::now().naive_utc(),
            pre_questionnaire: user.pre_questionnaire,
            post_questionnaire: None,
            browser: user.browser,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Weighting {
    pub token: String,
    pub fullscreen: bool,
    pub video_size: i32,
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
            fullscreen: weighting.fullscreen,
            video_size: i32::from(weighting.video_size),
            metric: weighting.metric,
            a: ObjectId::with_string(&weighting.a).unwrap(),
            b: ObjectId::with_string(&weighting.b).unwrap(),
            weight: weighting.weight,
            time: Utc::now().naive_utc(),
        }
    }
}

impl Into<model::Weighting> for Weighting {
    fn into(self) -> model::Weighting {
        if self.video_size < u32::min_value() as i32 {
            panic!("video_size paramter underflows u32");
        }

        model::Weighting {
            token: self.token,
            fullscreen: self.fullscreen,
            video_size: self.video_size as u16,
            metric: self.metric,
            a: self.a.to_hex(),
            b: self.b.to_hex(),
            weight: self.weight,
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

    db.collection(COLLECTION_USER).create_index(
        doc! { "public": 1 },
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
