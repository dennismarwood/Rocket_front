use rocket::response::{Redirect, Flash};
use rocket::request::{FlashMessage};
use rocket::http::{CookieJar};
use rocket_dyn_templates::{Template, context};
use rocket::form::{Form, Context, Contextual};
use chrono::{NaiveDateTime, NaiveDate};
use crate::errors::{DecipherResponseError, GetAndProcessError};
use crate::requests::{get_and_process_data, patch_value, post_value, post_value_get_status_code};
use crate::models::{PostForm, Post, NewTagForm, NewTag};
use crate::common::reqwest_client;
use std::collections::HashMap;
use crate::post::{get_exisiting_post, GetExisitingPostError};
use crate::models::{PasswordUpdate, Response, UserUpdates, UserWithoutPHC, Tag};

#[get("/")]
pub async fn get_tags(jar: &CookieJar<'_>, flash: Option<FlashMessage<'_>>) -> Template {
    match get_and_process_data::<Vec<Tag>>(jar, &"tags").await {
        Ok(tags) => {
            println!("tags: - {:?}", tags);
            return Template::render("tags", context!{tags});
        }
            
        Err(GetAndProcessError::DecipherResponseError(e)) => 
            match e {
                DecipherResponseError::UnexpectedResponseCode(sc) => 
                    match sc {
                        reqwest::StatusCode::UNAUTHORIZED => return Template::render("login", context! {}),
                        _ => return Template::render("error/bad_backend_response", context! {response: sc.as_str()}),
                    }
            },
        Err(e) => return Template::render("error/bad_backend_response", context! {response: e.to_string()})
    };
}

#[post("/", data="<tag_form>")]
pub async fn add_tag(jar: &CookieJar<'_>, tag_form: Form<NewTagForm>) -> Result<Redirect, Template> {
    let tag = NewTag {name: tag_form.name.clone()};
    match post_value(jar, "tags", tag).await {
        Ok(r) => {
            match r.status_code {
                //Some(201) => Ok(Redirect::to(format!("/tag/{}", r.location.unwrap_or(String::new())))),
                Some(201) | Some(409) => Ok(Redirect::to("/tag")),
                _ => {
                    let e = r.errors.unwrap_or("Expected error not present".into()).to_string();
                    Err(Template::render("error/bad_backend_response", context! {response: e}))
                },
            }
        },
        Err(e) => Err(Template::render("error/bad_backend_response", context! {response: e.to_string()})), 
    }
}