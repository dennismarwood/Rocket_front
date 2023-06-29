use rocket::http::{CookieJar};
use rocket_dyn_templates::{Template, context};
//use rocket::form::{Context};
use crate::models::{Post, PostAndTag};
use thiserror::Error;
use crate::errors::{GetAndProcessError, DecipherResponseError};
use crate::requests::{get_and_process_data};

#[derive(Error, Debug)]
pub enum GetExisitingPostError {
    #[error(transparent)]
    GetAndProcessError(#[from] GetAndProcessError),
    #[error("The backend unexpectedly returned an empty set of posts")]
    EmptyVec,
}

pub async fn get_exisiting_post(id: i32, jar: &CookieJar<'_>) -> Result<PostAndTag, GetExisitingPostError> {
    let pat: Vec<PostAndTag> = get_and_process_data(jar, &format!("posts/{}", id)).await?;

    match pat.first() {
        Some(pat) => Ok(pat.clone()),
        None => Err(GetExisitingPostError::EmptyVec)
    }   
}

#[get("/")]
pub async fn new_post()-> Template {
    //let p = Post::default();
    Template::render("post", context!{})
}

#[get("/<id>", rank=1)]
pub async fn existing_post(id: i32, jar: &CookieJar<'_>) -> Template { 
    match get_exisiting_post(id, jar).await {
        Ok(_p) => Template::render("post", context!{}),
        Err(e@GetExisitingPostError::EmptyVec) => Template::render("error/bad_backend_response", context! {response: e.to_string()}),
        Err(GetExisitingPostError::GetAndProcessError(e)) => 
            match e {
                GetAndProcessError::DecipherResponseError(e) => 
                    match e {
                        DecipherResponseError::UnexpectedResponseCode(sc) => 
                            match sc {
                                reqwest::StatusCode::NOT_FOUND => Template::render("error/404", context!{}),
                                _ => Template::render("error/bad_backend_response", context! {response: sc.as_str()})
                            }
                    },
                _ => Template::render("error/bad_backend_response", context! {response: e.to_string()}),           
            }
    }
}