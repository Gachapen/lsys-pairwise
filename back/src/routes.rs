use bson::{from_bson, to_bson, Bson};
use mongodb::{self, ThreadedClient};
use mongodb::db::ThreadedDatabase;
use rand::{thread_rng, Rng};
use rand::distributions::{IndependentSample, Range};
use rocket::{Route, State};
use rocket::http::{RawStr, Status};
use rocket::request::FromParam;
use rocket::response::{status, NamedFile};
use rocket_contrib::json::Json;
use std::path::{Path, PathBuf};

use db;
use model::{Gender, Metric, Weighting};
use uuid::Uuid;
use stats::{self, SampleWeight};
use serde_enum;

#[derive(Serialize, Deserialize, Debug)]
struct UserError {
    error: String,
    details: Option<String>,
}

impl UserError {
    fn new(error: &str) -> UserError {
        UserError {
            error: error.to_string(),
            details: None,
        }
    }

    #[allow(dead_code)]
    fn with_description(error: &str, description: String) -> UserError {
        UserError {
            error: error.to_string(),
            details: Some(description),
        }
    }
}

type UserErrorResponse = status::Custom<Json<UserError>>;

impl Into<UserErrorResponse> for UserError {
    fn into(self) -> UserErrorResponse {
        status::Custom(Status::BadRequest, Json(self))
    }
}

#[derive(Deserialize)]
struct User {
    age: u8,
    gender: Gender,
}

impl From<User> for db::User {
    fn from(user: User) -> db::User {
        db::User {
            age: i32::from(user.age),
            gender: user.gender,
            token: format!("{}", Uuid::new_v4().simple()),
        }
    }
}

impl<'r> FromParam<'r> for Metric {
    type Error = &'r RawStr;

    fn from_param(param: &'r RawStr) -> Result<Self, Self::Error> {
        match serde_enum::from_str(param) {
            Ok(value) => Ok(value),
            Err(_) => Err(param),
        }
    }
}

/// Get all of the routes
pub fn routes() -> Vec<Route> {
    routes![
        index,
        post_user,
        get_task,
        get_criteria_weights,
        get_video,
        post_weight
    ]
}

#[get("/")]
fn index() -> Json {
    Json(json!({
        "description": "lsys-pairwise server index",
        "links": [],
    }))
}

#[post("/user", data = "<user>")]
fn post_user(user: Json<User>, db_client: State<mongodb::Client>) -> Result<Json, Json> {
    let db_user: db::User = user.into_inner().into();
    let user_bson = to_bson(&db_user).unwrap();
    let user_doc = user_bson.as_document().unwrap();
    let insertion = db_client
        .db(db::NAME)
        .collection(db::COLLECTION_USER)
        .insert_one(user_doc.clone(), None)
        .expect("Failed inserting new user");

    if !insertion.acknowledged || insertion.inserted_id.is_none() {
        return Err(Json(json!({
            "error": "Failed inserting document"
        })));
    }

    Ok(Json(json!({ "token": db_user.token })))
}

#[derive(Serialize)]
struct Pair {
    a: String,
    b: String,
}

#[get("/task")]
fn get_task(db_client: State<mongodb::Client>) -> Result<Json<Vec<Pair>>, Json> {
    let sample_cursor = db_client
        .db(db::NAME)
        .collection(db::COLLECTION_SAMPLE)
        .find(None, None)
        .expect("Failed retrieving samples");

    let documents: Result<Vec<_>, _> = sample_cursor.collect();
    let documents = documents.expect("Failed retrieveing sample documents");

    let samples: Result<Vec<db::Sample>, _> = documents
        .into_iter()
        .map(|doc| from_bson(Bson::from(doc)))
        .collect();
    let samples = samples.expect("Failed deserializing documents");

    let num_samples = samples.len();
    let num_pairs = (num_samples * (num_samples - 1)) / 2;
    let mut pairs = Vec::with_capacity(num_pairs);
    for (i, sample_a) in samples.iter().enumerate() {
        for sample_b in samples.iter().skip(i + 1) {
            pairs.push(Pair {
                a: sample_a.name.clone(),
                b: sample_b.name.clone(),
            });
        }
    }

    assert_eq!(num_pairs, pairs.len());

    let chance_range = Range::new(0.0, 1.0);
    let mut rng = thread_rng();

    // Randomize pairings
    pairs = pairs
        .into_iter()
        .map(|pair| {
            let chance = chance_range.ind_sample(&mut rng);
            if chance > 0.5 {
                Pair {
                    a: pair.b,
                    b: pair.a,
                }
            } else {
                pair
            }
        })
        .collect();

    // Randomize pair ordering
    rng.shuffle(&mut pairs);

    Ok(Json(pairs))
}

#[get("/task/<token>/ranking/<metric>")]
fn get_criteria_weights(
    token: &RawStr,
    metric: Metric,
    db_client: State<mongodb::Client>,
) -> Result<Json<Vec<SampleWeight>>, Json> {
    Ok(Json(
        stats::calculate_sample_weights(token, &metric, &db_client),
    ))
}

#[get("/video/<file..>")]
fn get_video(file: PathBuf) -> Option<NamedFile> {
    NamedFile::open(Path::new("video/").join(file)).ok()
}

#[post("/weight", data = "<weighting>")]
fn post_weight(
    weighting: Json<Weighting>,
    db_client: State<mongodb::Client>,
) -> Result<Json, UserErrorResponse> {
    let db = db_client.db(db::NAME);

    if db.collection(db::COLLECTION_USER)
        .find_one(Some(doc!{ "token": &weighting.token }), None)
        .expect("Failed looking up user")
        .is_none()
    {
        return Err(UserError::new("User not registered").into());
    }

    if db.collection(db::COLLECTION_WEIGHT)
        .find_one(
            Some(doc!{
                "token": &weighting.token,
                "metric": to_bson(&weighting.metric).unwrap().as_str().unwrap(),
                "a": &weighting.a,
                "b": &weighting.b,
            }),
            None,
        )
        .expect("Failed looking up weight")
        .is_some()
    {
        return Err(UserError::new("Weight already registered").into());
    }

    let weight_bson = to_bson(&weighting.into_inner()).unwrap();
    let weight_doc = weight_bson.as_document().unwrap();

    let insertion = db.collection(db::COLLECTION_WEIGHT)
        .insert_one(weight_doc.clone(), None)
        .expect("Failed inserting new weight");

    if !insertion.acknowledged || insertion.inserted_id.is_none() {
        panic!(
            "Failed inserting new weight: Acknowleded: {}, ID: {:#?}",
            insertion.acknowledged,
            insertion.inserted_id,
        );
    }

    Ok(Json(json!({})))
}
