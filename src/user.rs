//use rocket::serde::json::{Value, Json};
use rocket::response::{Redirect, Flash};
use rocket::http::{CookieJar};
use rocket_dyn_templates::{Template, context};
use rocket::form::{Form, Context, Contextual};
use chrono::{NaiveDateTime, NaiveDate};
use crate::errors::{DecipherResponseError, GetAndProcessError};
use crate::requests::{get_and_process_data, patch_value, post_value, post_value_get_status_code};
use crate::models::{PostForm, Post};
use crate::common::{reqwest_client};
use std::collections::HashMap;
use crate::post::{get_exisiting_post, GetExisitingPostError};
use crate::models::{PasswordUpdate, Response, UserUpdates, UserWithoutPHC, Tag};
//use crate::{common::*, blog};

//use reqwest::header::CONTENT_TYPE;

#[get("/update_pw")]
pub async fn update_pw_template()-> Template {
    Template::render("update_pw", &Context::default())
}

/**
 Verify new pw fields match. Confirm existing pw is correct. Set user's pw to updated value. 
*/
#[post("/process_pw_update", data = "<pw_update>")]
pub async fn process_pw_update<'a>(pw_update: Form<Contextual<'a, PasswordUpdate<'_>>>, jar: &CookieJar<'_>) -> Result<Redirect, Template> {
    //If form was valid
    if let Some(ref value) = pw_update.value {
        //Confirm that user knows current password
        match post_value_get_status_code(jar, "users/confirm_pw", HashMap::from([("password", value.current_password)])).await {
            Ok(sc) => {
                match sc {
                    204 => (),
                    401 => return Err(Template::render("update_pw", context!{messages: 
                        ["The current account password you entered was incorrect.<br>Password <b>not</b> updated."]})),
                    _ => return Err(Template::render("error/bad_backend_response", context! {response: sc})),
                }
            },
            Err(e) => return Err(Template::render("error/bad_backend_response", context! {response: e.to_string()})), 
        }
        //Update user password
        match patch_value(jar, "users", HashMap::from([("phc", value.new_password)])).await {
            Ok(None) => return Ok(Redirect::to("/user")),
            Ok(Some(r)) => {
                match r.status_code {
                    Some(401) => return Err(Template::render("update_pw", context!{})),
                    _ => {
                        let e = r.errors.unwrap_or("Expected error not present".into()).to_string();
                        return Err(Template::render("error/bad_backend_response", context! {response: e}))
                    },
                }
            },
            Err(e) => return Err(Template::render("error/bad_backend_response", context! {response: e.to_string()})), 
        }
    }

    //Form had errors
    Err(Template::render("update_pw", &pw_update.context))
}

#[get("/<id>")]
pub async fn get_user_by_id(id: i32, jar: &CookieJar<'_>) -> Template {
    match get_and_process_data::<UserWithoutPHC>(jar, &format!("users/{}", id)).await {
        Ok(u) => return Template::render("user", context!{u, admin: true}),
        Err(e) => match e {
            GetAndProcessError::DecipherResponseError(e) => 
                match e {
                    DecipherResponseError::UnexpectedResponseCode(sc) => 
                        match sc {
                            reqwest::StatusCode:: UNAUTHORIZED=> return Template::render("login", context! {messages: ["You must be logged in to view a user profile."]}),
                            reqwest::StatusCode::FORBIDDEN => return Template::render("post", context! {messages: ["You lack necessary privileges to access that page."]}),
                            _ => return Template::render("error/bad_backend_response", context! {response: sc.as_str()})
                        }
                },
            _ => return Template::render("error/bad_backend_response", context! {response: e.to_string()}),
        }
    };
}

#[get("/post/<id>")]
pub async fn existing_post(id: i32, jar: &CookieJar<'_>) -> Template {
    let available_tags: Vec<Tag> = match get_and_process_data(jar, &"tags?step=100").await {
        Ok(t) => t,
        Err(e) => return Template::render("error/bad_backend_response", context! {response: e.to_string()})
    };
    match get_exisiting_post(id, jar).await {
        Ok(pat) => Template::render("admin/edit_post", context!{pat, available_tags}),
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

#[get("/post", rank=1)]
pub async fn new_post(jar: &CookieJar<'_>) -> Template {
    let available_tags: Vec<Tag> = match get_and_process_data(jar, &"tags?step=100").await {
        Ok(t) => t,
        Err(e) => return Template::render("error/bad_backend_response", context! {response: e.to_string()})
    };
    Template::render("admin/new_post", context!{available_tags})
}

#[post("/post", data="<post_form>")]
pub async fn process_post(jar: &CookieJar<'_>, mut post_form: Form<PostForm>) -> Result<Redirect, Template> {
    //Convert data to post
    let post = Post {
        id: post_form.id.take(),
        title: post_form.title.take(),
        content: post_form.content.take(),
        author: post_form.author.take(),
        created: post_form.created.as_deref().and_then(|d| NaiveDateTime::parse_from_str(d, "%Y-%m-%dT%H:%M:%S").ok()),
        last_updated: post_form.last_updated.as_deref().and_then(|d| NaiveDate::parse_from_str(d, "%Y-%m-%d").ok()),
    };

    //let post_id = post.id.map_or(String::new(), |id| format!("/{}", id.to_string()));
    let post_id = post.id.map_or(String::new(), |id| id.to_string());

    //If this is a new post there will be no id included in the form.
    match post.id {
        Some(_) => 
            match patch_value(jar, format!("posts/{}", &post_id).as_str(), post).await {
                Ok(None) => Ok(Redirect::to(uri!("/user"))),
                Ok(r) => Err(Template::render("error/bad_backend_response", context! {response: r})),
                Err(e) => Err(Template::render("error/bad_backend_response", context! {response: e.to_string()})),
            },
        None => 
            match post_value(jar, format!("posts{}", &post_id).as_str(), post).await {
                Ok(r) => {
                    match r.status_code {
                        Some(201) => Ok(Redirect::to(format!("/post/{}", r.location.unwrap_or(String::new())))),
                        _ => {
                            let e = r.errors.unwrap_or("Expected error not present".into()).to_string();
                            Err(Template::render("error/bad_backend_response", context! {response: e}))
                        },
                    }
                },
                Err(e) => Err(Template::render("error/bad_backend_response", context! {response: e.to_string()})), 
            },
    }

}

#[get("/")]
pub async fn get_user(jar: &CookieJar<'_>) -> Result<Template, Flash<Redirect>> {
    match get_and_process_data::<UserWithoutPHC>(jar, &"users").await {
        Ok(t) => return Ok(Template::render("user", context!{user: t})),
        Err(GetAndProcessError::DecipherResponseError(e)) => 
            match e {
                DecipherResponseError::UnexpectedResponseCode(sc) => 
                    match sc {
                        reqwest::StatusCode::UNAUTHORIZED => return Err(Flash::error(Redirect::to("/session/login"), "You must be logged in to view your user profile.")),
                        _ => return Ok(Template::render("error/bad_backend_response", context! {response: sc.as_str()})),
                    }
            },
        Err(e) => return Ok(Template::render("error/bad_backend_response", context! {response: e.to_string()}))
    };
}

#[post("/patch_user", data = "<user_input>")]
//Web forms have only get and post methods. The frontend will route to itself and then generate a patch request to the backend. 
pub async fn patch_user(user_input: Form<UserUpdates>, jar: &CookieJar<'_>) -> Template {
    match patch_value(jar, "users", user_input.into_inner()).await {
        Ok(None) => Template::render("user", context! {}),
        Ok(Some(r)) => {
            match r.status_code {
                Some(401) => {println!("401--"); Template::render("login", context! {messages: ["You must be logged in to edit a user profile."]})},
                Some(403) => Template::render("post", context! {messages: ["You lack necessary privileges to access that page."]}),
                _ => {
                    let e = r.errors.unwrap_or("Expected error not present".into()).to_string();
                    Template::render("error/bad_backend_response", context! {response: e})
                }
            }},
        Err(e) => Template::render("error/bad_backend_response", context! {response: e.to_string()}),
    }
}

#[post("/patch_user/<id>", data = "<user_input>")]
//Web forms have only get and post methods. The frontend will route to itself and then generate a patch request to the backend. 
pub async fn patch_user_by_id(id: i32, user_input: Form<UserUpdates>, jar: &CookieJar<'_>) -> Result<Redirect, Template> {
    let mut target_url : reqwest::Url = reqwest::Url::parse(&format!("http://back/users/{}", id)).unwrap();
    target_url.set_port(Some(8001)).map_err(|_| "cannot be base").unwrap();

    let my_client = reqwest_client(jar).unwrap();
    match my_client.patch(target_url)
        .json(&user_input.into_inner())
        .send()
        .await{
        Ok(response) => {
            match response.status() 
            {
                reqwest::StatusCode::NO_CONTENT => {
                    Ok(Redirect::to("/user"))
                }
                reqwest::StatusCode::UNAUTHORIZED => {
                    Ok(Redirect::to("/session/login"))
                }
                _ => {
                    Err(Template::render("error/500", context! {response: response.status().as_u16()}))
                }
            }
        },
        Err(e) => {
            let response = e.to_string();
            Err(Template::render("error/500", context! {response}))
        }
    }
}

#[get("/list_all_users")]
pub async fn list_all_users(jar: &CookieJar<'_>) -> Result<Template, Redirect> {
    let mut target_url : reqwest::Url = reqwest::Url::parse("http://back/users/list_of_all_users").unwrap();
    target_url.set_port(Some(8001)).map_err(|_| "cannot be base").unwrap();

    let my_client = reqwest_client(jar).unwrap();
    match my_client.get(target_url).send().await{
        Ok(response) => {
            match response.status() 
            {
                reqwest::StatusCode::OK => {
                    let r: Response = response.json::<Response>().await.unwrap();
                    let array_of_users = match r.data {
                        Some(u) => u,
                        None => return Ok(Template::render("admin/list_of_users", context!{r})),
                    };
                    let users: Vec<UserWithoutPHC> = serde_json::from_value(array_of_users).unwrap();
                    Ok(Template::render("admin/list_of_users", context!{users}))
                }
                reqwest::StatusCode::UNAUTHORIZED => {
                    Err(Redirect::to("/session/login"))
                }
                _ => {
                    Ok(Template::render("error/500", context! {response: response.status().as_u16()}))
                }
            }
        },
        Err(e) => {
            let response = e.to_string();
            Ok(Template::render("error/500", context! {response}))
        }
    }
}

#[delete("/<id>")]
pub async fn delete_user(id: i32, jar: &CookieJar<'_>) -> Result<Redirect, Template> {
    let mut target_url : reqwest::Url = reqwest::Url::parse(&format!("http://back/users/{}", id)).unwrap();
    target_url.set_port(Some(8001)).map_err(|_| "cannot be base").unwrap();

    let my_client = reqwest_client(jar).unwrap();
    match my_client.delete(target_url).send().await{
        Ok(response) => {
            match response.status() 
            {
                reqwest::StatusCode::NO_CONTENT => Ok(Redirect::to("/user")),
                //reqwest::StatusCode::NOT_FOUND => Ok(Redirect::to("/user")),
                _ => Err(Template::render("error/500", context! {response: response.status().as_u16()})),
            }
        }
        Err(e) => {
            let response = e.to_string();
            Err(Template::render("error/500", context! {response}))
        }
    }
}
