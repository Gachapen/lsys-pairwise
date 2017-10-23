use bson::to_bson;
use mongodb::{self, ThreadedClient};
use mongodb::db::ThreadedDatabase;
use rocket::{Route, State};
use rocket_contrib::json::Json;

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
    routes![index, post_user]
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
