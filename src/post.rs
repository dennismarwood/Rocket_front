use rocket::http::{CookieJar};
use rocket_dyn_templates::{Template, context};
//use rocket::form::{Context};
use crate::models::{Post, PostAndTag};
use thiserror::Error;
use crate::errors::{ProcessError};
use crate::requests::{get_and_process_data};

#[derive(Error, Debug)]
pub enum GetExisitingPostError {
    #[error(transparent)]
    ParseError(#[from] ProcessError),
    #[error("The backend unexpectedly returned an empty set of posts")]
    EmptyVec,
}

pub async fn get_exisiting_post(id: i32, jar: &CookieJar<'_>) -> Result<PostAndTag, GetExisitingPostError> {
    let v_pat: Vec<PostAndTag> = get_and_process_data(jar, &format!("posts/{}", id)).await?;

    match v_pat.first() {
        Some(pat) => Ok(pat.clone()),
        None => Err(GetExisitingPostError::EmptyVec)
    }   
}

#[get("/")]
pub async fn new_post()-> Template {
    let p = Post::default();
    Template::render("new_post", context!{p,})
}

#[get("/<id>", rank=1)]
pub async fn existing_post(id: i32, jar: &CookieJar<'_>) -> Template {
    match get_exisiting_post(id, jar).await {
        Ok(_p) => Template::render("post", context!{}),
        Err(e@GetExisitingPostError::EmptyVec) => Template::render("error/bad_backend_response", context! {response: e.to_string()}),
        Err(GetExisitingPostError::ParseError(e)) =>
            match e {
                ProcessError::BadResponse(sc) => 
                match sc.as_u16() {
                    404 => Template::render("error/404", context!{}),
                    _ => Template::render("error/bad_backend_response", context! {response: sc.as_str()})
                }                
                _ => Template::render("error/bad_backend_response", context! {response: e.to_string()}),
            }
    }
}