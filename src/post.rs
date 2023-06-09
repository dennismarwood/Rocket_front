//use rocket::serde::json::{Value};
//use rocket::response::{Redirect, Flash};
//use serde::{Serialize, Deserialize};
//use rocket::http::{CookieJar};
use rocket_dyn_templates::{Template, context};
//use rocket::form::{Context};
use crate::models::{Post};

#[get("/post")]
pub async fn new_post()-> Template {
    let p = Post::default();
    Template::render("new_post", context!{p,})
}

#[get("/post/<_id>", rank=1)]
pub async fn existing_post(_id: i32)-> Template {
    //Ask back end for post
    //let p = get_post(None).await;
    //Template::render("new_post", context!{p,})
    todo!()
}