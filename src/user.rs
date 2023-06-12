use rocket::serde::json::{Value};
use rocket::response::{Redirect, Flash};
use rocket::http::{CookieJar};
use rocket_dyn_templates::{Template, context};
use rocket::form::{Form, Context, Contextual};
use crate::requests::{ProcessError};

use crate::post::{get_exisiting_post, GetExisitingPostError};
use crate::models::{PasswordUpdate, Response, UserUpdates, UserWithoutPHC};
use crate::common::*;

use reqwest::header::CONTENT_TYPE;

#[get("/update_pw")]
pub async fn update_pw_template()-> Template {
    Template::render("update_pw", &Context::default())
}

#[post("/process_pw_update", data = "<pw_update>")]
pub async fn process_pw_update<'a>(pw_update: Form<Contextual<'a, PasswordUpdate<'_>>>, jar: &CookieJar<'_>) -> Result<Redirect, Template> {
    //If form was valid
    if let Some(ref value) = pw_update.value {
        let mut target_url : reqwest::Url = reqwest::Url::parse("http://back/users/confirm_pw").unwrap();
        target_url.set_port(Some(8001)).map_err(|_| "cannot be base").unwrap();

        let my_client = reqwest_client(jar).unwrap();

        //Verify that the existing pw is valid
        match my_client.post(target_url).header(CONTENT_TYPE, "text/plain").body(value.current_password.to_string()).send().await{
            Ok(response) => {
                match response.status().as_u16() {
                    200 => (),
                    401 => return Err(Template::render("update_pw", context!{bad_pw: "bad_pw",})),
                    _ => return Err(Template::render("error/500", context! {response: response.status().as_u16()})),
                }
            },
            Err(e) => return Err(Template::render("error/500", context! {response: e.to_string()})), //Back end did not respond
        }
        
        target_url = reqwest::Url::parse("http://back/users/confirm_pw").unwrap();
        target_url.set_port(Some(8001)).map_err(|_| "cannot be base").unwrap();
        //Update the user's pw
        match my_client.post(target_url).header(CONTENT_TYPE, "text/plain").body(value.current_password.to_string()).send().await{
            Ok(response) => {
                match response.status().as_u16() {
                    200 => return Ok(Redirect::to("/user")),
                    401 => return Err(Template::render("update_pw", context!{bad_pw: "bad_pw",})),
                    _ => return Err(Template::render("error/500", context! {response: response.status().as_u16()})),
                }
            },
            Err(e) => return Err(Template::render("error/500", context! {response: e.to_string()})) //Back end did not respond
        }
    }

    //Form had errors
    Err(Template::render("update_pw", &pw_update.context))
}

#[get("/<id>")]
pub async fn get_user_by_id(id: i32, jar: &CookieJar<'_>) -> Result<Template, Flash<Redirect>> {
    let mut target_url: reqwest::Url = reqwest::Url::parse(&format!("http://back/users/{}", id)).unwrap();
    target_url.set_port(Some(8001)).map_err(|_| "cannot be base").unwrap();

    let my_client = reqwest_client(jar).unwrap();

    match my_client.get(target_url).send().await{
        Ok(response) => {
            match response.status() {
                reqwest::StatusCode::OK => {
                    let r: Response = response.json::<Response>().await.unwrap();
                    match r.data {
                        Some(user) => return Ok(Template::render("user", context!{user, admin: true})),
                        None => return Ok(Template::render("error/500", context! {r})),//Looked up a user but did not get user data.
                    };
                }
                reqwest::StatusCode::INTERNAL_SERVER_ERROR => { //Backend returns a 500
                    let response: Value = serde_json::from_str(&response.text().await.unwrap()[..]).unwrap();
                    Ok(Template::render("error/500", context! {response}))
                }
                reqwest::StatusCode::UNAUTHORIZED => {
                    Err(Flash::error(Redirect::to("/session/login"), "You must be logged in to view your user profile."))
                }
                _ => { //Backend returned status code other than 200, 500, 401
                    Ok(Template::render("error/500", context! {response: response.status().as_u16()}))
                }
            }
        },
        Err(e) => {//Frontend returns a 500
            let response = e.to_string();
            Ok(Template::render("error/500", context! {response}))
        }
    }
}

#[get("/post/<id>")]
pub async fn existing_post(id: i32, jar: &CookieJar<'_>) -> Template {
    match get_exisiting_post(id, jar).await {
        Ok(_p) => Template::render("edit_post", context!{}),
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

#[get("/post", rank=1)]
pub async fn new_post(jar: &CookieJar<'_>) -> Template {
    todo!();
}

#[get("/")]
pub async fn get_user(jar: &CookieJar<'_>) -> Result<Template, Flash<Redirect>> {
    let mut target_url : reqwest::Url = reqwest::Url::parse("http://back/users").unwrap();
    target_url.set_port(Some(8001)).map_err(|_| "cannot be base").unwrap();

    let my_client = reqwest_client(jar).unwrap();

    match my_client.get(target_url).send().await{
        Ok(response) => {
            match response.status() {
                reqwest::StatusCode::OK => {
                    let r: Response = response.json::<Response>().await.unwrap();
                    match r.data {
                        Some(user) => return Ok(Template::render("user", context!{user})),
                        None => return Ok(Template::render("error/500", context! {r})),//Looked up a user but did not get user data.
                    };
                }
                reqwest::StatusCode::INTERNAL_SERVER_ERROR => { //Backend returns a 500
                    let response: Value = serde_json::from_str(&response.text().await.unwrap()[..]).unwrap();
                    Ok(Template::render("error/500", context! {response}))
                }
                reqwest::StatusCode::UNAUTHORIZED => {
                    Err(Flash::error(Redirect::to("/session/login"), "You must be logged in to view your user profile."))
                }
                _ => { //Backend returned status code other than 200, 500, 401
                    let response: Value = serde_json::from_str(&response.text().await.unwrap()[..]).unwrap();
                    Ok(Template::render("error/default", context! {response}))
                }
            }
        },
        Err(e) => {//Frontend returns a 500
            let response = e.to_string();
            Ok(Template::render("error/500", context! {response}))
        }
    }
}

#[post("/patch_user", data = "<user_input>")]
//Web forms have only get and post methods. The frontend will route to itself and then generate a patch request to the backend. 
pub async fn patch_user(user_input: Form<UserUpdates>, jar: &CookieJar<'_>) -> Result<Redirect, Template> {
    let mut target_url : reqwest::Url = reqwest::Url::parse("http://back/users").unwrap();
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
