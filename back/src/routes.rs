use bson::{from_bson, to_bson, Bson};
use bson::oid::{self, ObjectId};
use chrono::Utc;
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
use std::error::Error;
use std::path::Path;

use db;
use model::{Gender, Metric, Sample, Weighting};
use uuid::Uuid;
use stats::{self, SampleWeight};
use serde_enum;

#[derive(Debug)]
struct RequestError {
    status: Status,
    error: String,
    details: Option<String>,
}

impl RequestError {
    fn new(error: &str) -> RequestError {
        RequestError {
            status: Status::BadRequest,
            error: error.to_string(),
            details: None,
        }
    }

    fn with_status(status: Status, error: &str) -> RequestError {
        RequestError {
            status: status,
            error: error.to_string(),
            details: None,
        }
    }

    #[allow(dead_code)]
    fn with_description(status: Status, error: &str, description: String) -> RequestError {
        RequestError {
            status: status,
            error: error.to_string(),
            details: Some(description),
        }
    }
}

type RequestErrorResponse = status::Custom<Json<RequestError>>;

impl Into<RequestErrorResponse> for RequestError {
    fn into(self) -> RequestErrorResponse {
        status::Custom(self.status, Json(self))
    }
}

#[derive(Deserialize)]
struct User {
    age: u8,
    gender: Gender,
    task: String,
}

impl From<User> for db::User {
    fn from(user: User) -> db::User {
        db::User {
            age: i32::from(user.age),
            gender: user.gender,
            token: format!("{}", Uuid::new_v4().simple()),
            task: user.task,
            register_date: Utc::now().naive_utc(),
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
        post_weight,
        get_sample,
        get_technical_ranking,
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
fn post_user(
    user: Json<User>,
    db_client: State<mongodb::Client>,
) -> Result<Json, RequestErrorResponse> {
    let db_user: db::User = user.into_inner().into();

    let sample_res = db_client
        .db(db::NAME)
        .collection(db::COLLECTION_SAMPLE)
        .find_one(
            Some(doc! {
                "task": &db_user.task,
            }),
            None,
        )
        .expect("Failed retrieving samples");

    if sample_res.is_none() {
        return Err(RequestError::new("Task not found").into());
    }

    let user_bson = to_bson(&db_user).unwrap();
    let user_doc = user_bson.as_document().unwrap();
    let insertion = db_client
        .db(db::NAME)
        .collection(db::COLLECTION_USER)
        .insert_one(user_doc.clone(), None)
        .expect("Failed inserting new user");

    if let Some(write_exception) = insertion.write_exception {
        panic!(
            "Failed inserting new user: {}",
            write_exception.description()
        );
    }

    if !insertion.acknowledged {
        panic!("Failed inserting new user: Insertion not acknowleded");
    }

    if insertion.inserted_id.is_none() {
        panic!("Failed inserting new user: Not ID in result");
    }

    Ok(Json(json!({ "token": db_user.token })))
}

#[derive(Serialize)]
struct Pair {
    a: String,
    b: String,
}

#[get("/task/<user>")]
fn get_task(
    user: &RawStr,
    db_client: State<mongodb::Client>,
) -> Result<Json<Vec<Pair>>, RequestErrorResponse> {
    let task = get_users_task(user, &db_client)?;
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

#[get("/ranking/<user>/<metric>")]
fn get_criteria_weights(
    user: &RawStr,
    metric: Metric,
    db_client: State<mongodb::Client>,
) -> Result<Json<Vec<SampleWeight>>, RequestErrorResponse> {
    let task = get_users_task(user, &db_client)?;

    Ok(Json(stats::calculate_sample_weights(
        &task,
        user,
        &metric,
        &db_client,
    )))
}

#[get("/task/<task>/ranking/technical")]
fn get_technical_ranking(
    task: &RawStr,
    db_client: State<mongodb::Client>,
) -> Result<Json<Vec<SampleWeight>>, RequestErrorResponse> {
    let sample_cursor = db_client
        .db(db::NAME)
        .collection(db::COLLECTION_SAMPLE)
        .find(
            Some(doc! {
                "task": task.as_str(),
            }),
            None,
        )
        .expect("Failed retrieving samples");

    let documents: Result<Vec<_>, _> = sample_cursor.collect();
    let documents = documents.expect("Failed retrieveing sample documents");

    let samples: Vec<(String, Sample)> = documents
        .into_iter()
        .map(|doc| {
            (
                doc.get_object_id("_id").unwrap().to_hex(),
                from_bson(Bson::from(doc)).unwrap(),
            )
        })
        .collect();

    let mut weights: Vec<_> = samples
        .into_iter()
        .map(|(id, sample)| {
            SampleWeight {
                name: id,
                weight: sample.fitness,
            }
        })
        .collect();

    let normalizer = 1.0 / weights.iter().map(|weight| weight.weight).sum::<f32>();
    for weight in &mut weights {
        weight.weight = weight.weight * normalizer;
    }
    weights.sort_by(|a, b| b.weight.partial_cmp(&a.weight).unwrap());

    Ok(Json(weights))
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
        let sample: Sample = from_bson(Bson::from(doc)).expect("Failed deserializing doc");
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
) -> Result<Json, RequestErrorResponse> {
    let db = db_client.db(db::NAME);

    if db.collection(db::COLLECTION_USER)
        .find_one(Some(doc!{ "token": &weighting.token }), None)
        .expect("Failed looking up user")
        .is_none()
    {
        return Err(RequestError::new("User not registered").into());
    }

    if db.collection(db::COLLECTION_WEIGHT)
        .find_one(
            Some(doc!{
                "token": &weighting.token,
                "metric": serde_enum::to_string(&weighting.metric).unwrap(),
                "a": ObjectId::with_string(&weighting.a).unwrap(),
                "b": ObjectId::with_string(&weighting.b).unwrap(),
            }),
            None,
        )
        .expect("Failed looking up weight")
        .is_some()
    {
        return Err(RequestError::new("Weight already registered").into());
    }

    let weight_bson = to_bson(&weighting.into_inner()).unwrap();
    let weight_doc = weight_bson.as_document().unwrap();

    let insertion = db.collection(db::COLLECTION_WEIGHT)
        .insert_one(weight_doc.clone(), None)
        .expect("Failed inserting new weight");

    if let Some(write_exception) = insertion.write_exception {
        panic!(
            "Failed inserting new weight: {}",
            write_exception.description()
        );
    }

    if !insertion.acknowledged {
        panic!("Failed inserting new weight: Insertion not acknowleded");
    }

    if insertion.inserted_id.is_none() {
        panic!("Failed inserting new weight: Not ID in result");
    }

    Ok(Json(json!({})))
}

#[get("/sample/<id>")]
fn get_sample(
    id: &RawStr,
    db_client: State<mongodb::Client>,
) -> Result<Json<Sample>, RequestErrorResponse> {
    let db = db_client.db(db::NAME);

    let object_id = match ObjectId::with_string(id) {
        Ok(object_id) => object_id,
        Err(error) => match error {
            oid::Error::ArgumentError(_) | oid::Error::FromHexError(_) => {
                return Err(RequestError::new("Invalid ID provided").into());
            }
            _ => panic!("Failed creating OID: {}", error.description()),
        },
    };

    let sample_res = db.collection(db::COLLECTION_SAMPLE)
        .find_one(
            Some(doc!{
                "_id": object_id,
            }),
            None,
        )
        .expect("Failed looking up sample");

    if let Some(sample_doc) = sample_res {
        let sample = from_bson(Bson::from(sample_doc)).expect("Failed deserializing Sample");
        Ok(Json(sample))
    } else {
        return Err(RequestError::with_status(Status::NotFound, "Sample not found").into());
    }
}

fn get_users_task(
    user_token: &str,
    db_client: &mongodb::Client,
) -> Result<String, RequestErrorResponse> {
    let user_res = db_client
        .db(db::NAME)
        .collection(db::COLLECTION_USER)
        .find_one(
            Some(doc! {
                "token": user_token,
            }),
            None,
        )
        .expect("Failed retrieving user");

    match user_res {
        Some(user_doc) => {
            let user: db::User =
                from_bson(Bson::from(user_doc)).expect("Failed deserializing User");
            Ok(user.task)
        }
        None => Err(RequestError::with_status(Status::NotFound, "User not found").into()),
    }
}
