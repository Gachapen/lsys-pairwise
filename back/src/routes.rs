use bson::{from_bson, to_bson, Bson};
use bson::oid::{self, ObjectId};
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
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::path::Path;

use db;
use model::{Metric, PostQuestionnaire, PreQuestionnaire, Sample, User, Weighting};
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

    #[allow(dead_code)]
    fn with_status(status: Status, error: &str) -> RequestError {
        RequestError {
            status: status,
            error: error.to_string(),
            details: None,
        }
    }

    fn not_found(error: &str) -> RequestError {
        RequestError {
            status: Status::NotFound,
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
        get_tasks,
        get_task,
        get_criteria_weights,
        get_video,
        post_weight,
        get_sample,
        get_technical_ranking,
        put_pre_questionnaire,
        put_post_questionnaire,
        get_user_task,
        get_user_public,
        get_user_source,
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

#[put("/user/<user_token>/pre", data = "<questionnaire>")]
fn put_pre_questionnaire(
    user_token: &RawStr,
    questionnaire: Json<PreQuestionnaire>,
    db_client: State<mongodb::Client>,
) -> Result<(), RequestErrorResponse> {
    let questionnaire_doc = to_bson(&questionnaire.into_inner()).unwrap();
    let update_res = db_client
        .db(db::NAME)
        .collection(db::COLLECTION_USER)
        .update_one(
            doc! {
                "token": user_token.as_str(),
            },
            doc! {
                "$set": {
                    "pre_questionnaire": questionnaire_doc,
                },
            },
            None,
        )
        .expect("Failed updating user");

    if let Some(write_exception) = update_res.write_exception {
        panic!("Failed updating user: {}", write_exception.description());
    }

    if !update_res.acknowledged {
        panic!("Failed updating user: Update not acknowleded");
    }

    if update_res.matched_count == 0 {
        return Err(RequestError::not_found("User not found").into());
    }

    Ok(())
}

#[put("/user/<user_token>/post", data = "<questionnaire>")]
fn put_post_questionnaire(
    user_token: &RawStr,
    questionnaire: Json<PostQuestionnaire>,
    db_client: State<mongodb::Client>,
) -> Result<(), RequestErrorResponse> {
    let questionnaire_doc = to_bson(&questionnaire.into_inner()).unwrap();
    let update_res = db_client
        .db(db::NAME)
        .collection(db::COLLECTION_USER)
        .update_one(
            doc! {
                "token": user_token.as_str(),
            },
            doc! {
                "$set": {
                    "post_questionnaire": questionnaire_doc,
                },
            },
            None,
        )
        .expect("Failed updating user");

    if let Some(write_exception) = update_res.write_exception {
        panic!("Failed updating user: {}", write_exception.description());
    }

    if !update_res.acknowledged {
        panic!("Failed updating user: Update not acknowleded");
    }

    if update_res.matched_count == 0 {
        return Err(RequestError::not_found("User not found").into());
    }

    Ok(())
}

#[get("/user/<user_token>/task")]
fn get_user_task(
    user_token: &RawStr,
    db_client: State<mongodb::Client>,
) -> Result<Json, RequestErrorResponse> {
    let user_res = db_client
        .db(db::NAME)
        .collection(db::COLLECTION_USER)
        .find_one(
            Some(doc! {
                "token": user_token.as_str(),
            }),
            Some(FindOptions {
                projection: Some(doc! {
                    "task": 1,
                    "_id": 0,
                }),
                ..Default::default()
            }),
        )
        .expect("Failed finding user");

    let user_doc = match user_res {
        Some(doc) => doc,
        None => return Err(RequestError::not_found("User not found").into()),
    };

    let task = user_doc
        .get_str("task")
        .expect("User did not have 'task' field");

    Ok(Json(json!({
        "task": task.to_string()
    })))
}

#[get("/user/<user_token>/public")]
fn get_user_public(
    user_token: &RawStr,
    db_client: State<mongodb::Client>,
) -> Result<Json, RequestErrorResponse> {
    let user_res = db_client
        .db(db::NAME)
        .collection(db::COLLECTION_USER)
        .find_one(
            Some(doc! {
                "token": user_token.as_str(),
            }),
            Some(FindOptions {
                projection: Some(doc! {
                    "public": 1,
                    "_id": 0,
                }),
                ..Default::default()
            }),
        )
        .expect("Failed finding user");

    let user_doc = match user_res {
        Some(doc) => doc,
        None => return Err(RequestError::not_found("User not found").into()),
    };

    let public = user_doc
        .get_str("public")
        .expect("User did not have 'public' field");

    Ok(Json(json!({
        "public": public.to_string()
    })))
}

#[get("/user/<user_token>/source")]
fn get_user_source(
    user_token: &RawStr,
    db_client: State<mongodb::Client>,
) -> Result<Json, RequestErrorResponse> {
    let user_res = db_client
        .db(db::NAME)
        .collection(db::COLLECTION_USER)
        .find_one(
            Some(doc! {
                "token": user_token.as_str(),
            }),
            Some(FindOptions {
                projection: Some(doc! {
                    "source": 1,
                    "_id": 0,
                }),
                ..Default::default()
            }),
        )
        .expect("Failed finding user");

    let user_doc = match user_res {
        Some(doc) => doc,
        None => return Err(RequestError::not_found("User not found").into()),
    };

    let source = user_doc
        .get_str("source")
        .expect("User did not have 'source' field");

    Ok(Json(json!({
        "source": source.to_string()
    })))
}

#[get("/task")]
fn get_tasks(db_client: State<mongodb::Client>) -> Result<Json<Vec<String>>, RequestErrorResponse> {
    let task_bsons = db_client
        .db(db::NAME)
        .collection(db::COLLECTION_SAMPLE)
        .distinct("task", None, None)
        .expect("Failed retrieving samples");

    let tasks: Vec<String> = task_bsons
        .iter()
        .map(|bson_value| bson_value.as_str())
        .collect::<Option<Vec<&str>>>()
        .expect("Some task fields were not strings")
        .into_iter()
        .map(|s| s.to_string())
        .collect();

    Ok(Json(tasks))
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

    let weights_cursor = db_client
        .db(db::NAME)
        .collection(db::COLLECTION_WEIGHT)
        .find(
            Some(doc! {
                "token": user.as_str(),
                "metric": serde_enum::to_string(&Metric::Pleasing).unwrap(),
            }),
            Some(FindOptions {
                projection: Some(doc! {
                    "_id": 0,
                    "a": 1,
                    "b": 1,
                }),
                ..Default::default()
            }),
        )
        .expect("Failed retrieving weights");

    let mut weighted: HashMap<ObjectId, HashSet<ObjectId>> = HashMap::new();
    for weight_doc in weights_cursor {
        let weight_doc = weight_doc.expect("Failed retrieving weight");
        let a = weight_doc.get_object_id("a").unwrap();
        let b = weight_doc.get_object_id("b").unwrap();

        weighted
            .entry(a.clone())
            .or_insert_with(HashSet::new)
            .insert(b.clone());
        weighted
            .entry(b.clone())
            .or_insert_with(HashSet::new)
            .insert(a.clone());
    }

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

    let sample_documents: Result<Vec<_>, _> = sample_cursor.collect();
    let sample_documents = sample_documents.expect("Failed retrieveing sample documents");

    let sample_ids: Vec<&ObjectId> = sample_documents
        .iter()
        .map(|doc| doc.get_object_id("_id"))
        .collect::<Result<_, _>>()
        .expect("Failed deserializing documents");

    let num_samples = sample_ids.len();
    let num_pairs = (num_samples * (num_samples - 1)) / 2;
    let mut pairs = Vec::with_capacity(num_pairs);
    for (i, id_a) in sample_ids.iter().enumerate() {
        for id_b in sample_ids.iter().skip(i + 1) {
            // Only keep pairs that have not been weighted before
            let needs_measurement = match weighted.get(id_a) {
                None => true,
                Some(paired) => !paired.contains(id_b),
            };

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
    let weights = stats::calculate_sample_weights(&task, user, &metric, &db_client);

    if let Ok(weights) = weights {
        Ok(Json(weights))
    } else {
        Err(RequestError::not_found("Missing weights").into())
    }
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
        weight.weight *= normalizer;
    }
    weights.sort_by(|a, b| b.weight.partial_cmp(&a.weight).unwrap());

    Ok(Json(weights))
}

#[get("/video/<id>/<ext>")]
fn get_video(id: &RawStr, ext: &RawStr, db_client: State<mongodb::Client>) -> Option<NamedFile> {
    let object_id = match ObjectId::with_string(id) {
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

    let db_weighting: db::Weighting = weighting.into_inner().into();
    let weight_bson = to_bson(&db_weighting).unwrap();
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
        return Err(RequestError::not_found("Sample not found").into());
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
        None => Err(RequestError::not_found("User not found").into()),
    }
}
