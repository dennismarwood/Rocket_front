//use rocket::serde::json::{Value};
//use rocket::response::{Redirect, Flash};
//use serde::{Serialize, Deserialize};
//use rocket::http::{CookieJar};
use rocket_dyn_templates::{Template, context};
use rocket::form::{Form, Context, Contextual};

#[get("/new_post")]
pub async fn new_post()-> Template {
    Template::render("new_post", &Context::default())
}