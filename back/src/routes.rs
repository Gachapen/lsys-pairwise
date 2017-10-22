use rocket::Route;
use rocket_contrib::json::Json;

/// Get all of the routes
pub fn routes() -> Vec<Route> {
    routes![index]
}

#[get("/")]
fn index() -> Json {
    Json(json!({
        "description": "lsys-pairwise server index",
        "links": [],
    }))
}
