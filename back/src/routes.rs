use bson::{from_bson, to_bson, Bson};
use mongodb::{self, ThreadedClient};
use mongodb::db::ThreadedDatabase;
use rocket::{Route, State};
use rocket::response::NamedFile;
use rocket_contrib::json::Json;
use std::path::{Path, PathBuf};

use db;
use model::Gender;
use uuid::Uuid;

#[derive(Deserialize)]
struct User {
    age: u8,
    gender: Gender,
}

impl From<User> for db::User {
    fn from(user: User) -> db::User {
        db::User {
            age: user.age as i32,
            gender: user.gender,
        }
    }
}

/// Get all of the routes
pub fn routes() -> Vec<Route> {
    routes![index, post_user, get_task, get_video]
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

    if !insertion.acknowledged && insertion.inserted_id.is_none() {
        return Err(Json(json!({
            "error": "Failed inserting document"
        })));
    }

    let token = format!("{}", Uuid::new_v4().simple());

    Ok(Json(json!({ "token": token })))
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
        .iter()
        .map(|doc| from_bson(Bson::from(doc.clone())))
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

    Ok(Json(pairs))
}

#[get("/video/<file..>")]
fn get_video(file: PathBuf) -> Option<NamedFile> {
    NamedFile::open(Path::new("video/").join(file)).ok()
}
