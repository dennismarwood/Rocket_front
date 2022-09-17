#[macro_use]
extern crate rocket;

mod auth;

use auth::BasicAuth;
use rocket::{Build, Rocket, response::status};
use rocket::serde::json::{Json, Value, json, self};
//use rocket::http::Status;
//use rocket::request::{Request, FromRequest, Outcome};

#[derive(FromForm)]
struct Filters {
    age: u8,
    active: bool,
}

#[route(GET, uri = "/user/<uuid>", rank = 1, format = "text/plain")]
fn user(uuid: &str) -> String{
    format!("Your uuid: {}", uuid)
}

#[route(GET, uri = "/user/<uuid>", rank = 2)]
fn usera(uuid: &str) -> String{
    format!("usera Your uuid: {}", uuid)
}

#[route(GET, uri = "/users/<grade>?<filters..>")]
fn users(grade: u8, filters: Filters) -> String{
    format!("grade: {}, age: {}, active: {}", grade, filters.age, filters.active)
}

#[get("/")] //This is a macro attribute
fn index() -> &'static str {
    "Hello, from Rocket!!!~\n"
}

#[get("/blog")]
fn blog() -> &'static str {
    "Blog"
}

#[get("/hire/<_..>")]
fn hire() -> &'static str {
    "Hire"
}

#[get("/mail/<_..>")]
fn mail() -> &'static str {
    "Mail"
}

#[get("/portfolio/<_..>")]
fn portfolio() -> &'static str {
    "Portfolio"
}

#[get("/resume/<_..>")]
fn resume() -> &'static str {
    "Resume"
}

#[get("/foo/<_>/bar")]
fn foo_bar() -> &'static str {
    "Match on /foo/*/bar"
}

#[get("/foo/<_..>/bar/<_..>", rank=2)]
fn foo_any_bar() -> &'static str {
    "Match on /foo/*/bar or /foo/*/*/bar or /foo/*/bar/* or /foo/*/bar/*/*/"
}

#[get("/user/<id>")]
fn user_dup(id: usize) { /* ... */ }

#[get("/user/<id>", rank = 2)]
fn user_dup_int(id: isize) { /* ... */ }

#[get("/user/<id>", rank = 3)]
fn user_dup_str(id: &str) { /* ... */ }

#[get("/myjson")]
fn myjson() -> Value {
    json!("Hello, from Rocket! JSON")
}

#[get("/rustaceans")]
fn get_rustaceans(auth: BasicAuth) -> Value {
        json!([{ "id": 1, "name": "John Doe" }, { "id": 2, "name": "John Doe again" }])
}

#[get("/rustaceans/<id>")]
fn view_rustacean(id: i32, _auth: BasicAuth) -> Value {
    json!({"id": id, "name": "John Doe", "email": "john@doe.com"})
}

#[post("/rustaceans", format = "json")]
fn create_rustacean(_auth: BasicAuth) -> Value {
    json!({"id": 3, "name": "John Doe", "email": "john@doe.com"})
}

#[put("/rustaceans/<id>", format = "json")]
fn update_rustacean(id: i32, _auth: BasicAuth) -> Value {
    json!({"id": id, "name": "John Doe", "email": "john@doe.com"})
}

#[delete("/rustaceans/<_id>")]
fn delete_rustacean(_id: i32, _auth: BasicAuth) -> status::NoContent {
    status::NoContent
}

#[catch(404)]
fn not_found() -> Value {
    json!("Not Found!")
}

#[catch(401)]
fn unauthorized() -> Value {
    json!("Unauthorized")
}

#[catch(422)]
fn unprocessable_entity() -> Value {
    json!("422 - Unprocessable Entity. Verify that submitted data is valid.")
}

#[rocket::async_trait]
impl<'r> auth::FromRequest<'r> for BasicAuth {
    type Error = ();

    async fn from_request(request: &'r auth::Request<'_>) -> auth::Outcome<Self, Self::Error> {
        let auth_header = request.headers().get_one("Authorization");
        if let Some(auth_header) = auth_header {
            if let Some(auth) = Self::from_authorization_header(auth_header) {
                if auth.username == String::from("foo") && auth.password == String::from("bar") {
                    return auth::Outcome::Success(auth)
                }
            }
        }

        auth::Outcome::Failure((auth::Status::Unauthorized, ()))
    }
}

#[launch] //Will genererate the main function
fn rocket() -> Rocket<Build> { //Built the rocket here
    rocket::build()
        .mount("/udemy/2/4", routes![myjson])
        .mount("/udemy/2/5", routes![get_rustaceans,create_rustacean,view_rustacean,update_rustacean,delete_rustacean])
        .mount("/udemy/3/8", routes![])
        //.mount("/", routes![send_])
        .mount("/", routes![blog])
        .mount("/", routes![hire])
        .mount("/", routes![mail])
        .mount("/", routes![portfolio])
        .mount("/", routes![resume])
        .mount("/", routes![index, user, users, usera, foo_bar, foo_any_bar])
        .mount("/forwards_example", routes![])
        .register("/udemy/2/5", catchers![not_found, unauthorized, unprocessable_entity])
}