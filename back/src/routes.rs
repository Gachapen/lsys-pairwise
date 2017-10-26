use bson::{from_bson, to_bson, Bson};
use bson::oid::ObjectId;
use mongodb::{self, ThreadedClient};
use mongodb::db::ThreadedDatabase;
use mongodb::coll::options::FindOptions;
use rand::{thread_rng, Rng};
use rand::distributions::{IndependentSample, Range};
use rocket::{Route, State};
use rocket::http::{RawStr, Status};
use rocket::request::FromParam;
use rocket::response::{status, NamedFile};
use rocket_contrib::json::Json;
use std::path::Path;

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

#[get("/task/<task>/user/<user>")]
fn get_task(
    task: &RawStr,
    user: &RawStr,
    db_client: State<mongodb::Client>,
) -> Result<Json<Vec<Pair>>, Json> {
    let sample_cursor = db_client
        .db(db::NAME)
        .collection(db::COLLECTION_SAMPLE)
        .find(
            Some(doc! {
                "task": task.as_str(),
            }),
            Some(FindOptions {
                projection: Some(doc! {
                    "_id": 1,
                }),
                ..Default::default()
            }),
        )
        .expect("Failed retrieving samples");

    let documents: Result<Vec<_>, _> = sample_cursor.collect();
    let documents = documents.expect("Failed retrieveing sample documents");

    let ids: Vec<&ObjectId> = documents
        .iter()
        .map(|doc| doc.get_object_id("_id"))
        .collect::<Result<_, _>>()
        .expect("Failed deserializing documents");

    let num_samples = ids.len();
    let num_pairs = (num_samples * (num_samples - 1)) / 2;
    let mut pairs = Vec::with_capacity(num_pairs);
    for (i, id_a) in ids.iter().enumerate() {
        for id_b in ids.iter().skip(i + 1) {
            let needs_measurement =
                db::missing_measurement(user, id_a, id_b, &db_client).expect("Failed quering DB");
            if needs_measurement {
                pairs.push(Pair {
                    a: id_a.to_hex(),
                    b: id_b.to_hex(),
                });
            }
        }
    }

    assert!(pairs.len() <= num_pairs);

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

#[get("/task/<task>/ranking/<metric>/user/<user>")]
fn get_criteria_weights(
    task: &RawStr,
    metric: Metric,
    user: &RawStr,
    db_client: State<mongodb::Client>,
) -> Result<Json<Vec<SampleWeight>>, Json> {
    Ok(Json(stats::calculate_sample_weights(
        task,
        user,
        &metric,
        &db_client,
    )))
}

#[get("/video/<id>/<ext>")]
fn get_video(id: &RawStr, ext: &RawStr, db_client: State<mongodb::Client>) -> Option<NamedFile> {
    let object_id = match ObjectId::with_string(&id) {
        Ok(object_id) => object_id,
        Err(_) => return None,
    };

    let doc = db_client
        .db(db::NAME)
        .collection(db::COLLECTION_SAMPLE)
        .find_one(
            Some(doc! {
                "_id": object_id,
            }),
            None,
        )
        .expect("Failed retrieving samples");

    if let Some(doc) = doc {
        let sample: db::Sample = from_bson(Bson::from(doc)).expect("Failed deserializing doc");
        let filename = format!("{}.{}", sample.name, ext);
        let path = Path::new("task/").join(sample.task).join(filename);
        println!("{}", path.to_str().unwrap());
        NamedFile::open(path).ok()
    } else {
        None
    }
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
                "a": ObjectId::with_string(&weighting.a).unwrap(),
                "b": ObjectId::with_string(&weighting.b).unwrap(),
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
