//use rocket::serde::json::{Value};
//use rocket::response::{Redirect, Flash};
//use serde::{Serialize, Deserialize};
//use rocket::http::{CookieJar};
use rocket_dyn_templates::{Template};
use rocket::form::{Context};


#[derive(Serialize, Deserialize, Debug)]
pub enum PostFields {
    Id(i32),
    Title(String),
    Author(String),
    Created(chrono::NaiveDateTime),
    LastUpdated(chrono::NaiveDate),
    Content(String,)
}

#[get("/post")]
pub async fn new_post()-> Template {
    Template::render("new_post", &Context::default())
}

#[get("/post/<_id>", rank=2)]
pub async fn existing_post(_id: i32)-> Template {
    Template::render("new_post", &Context::default())
}