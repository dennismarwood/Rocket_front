use rocket::serde::json::{Value};
use rocket::response::{Redirect, Flash};
use rocket::http::{CookieJar};
use rocket_dyn_templates::{Template, context};
use rocket::form::{Form, Context, Contextual};

use crate::models::{PasswordUpdate, Response, UserUpdates, UserWithoutPHC};

pub mod routes {
    use reqwest::header::CONTENT_TYPE;

    use super::*;

    pub fn reqwest_client<'a>(jar: &CookieJar<'_>) -> Result<reqwest::Client, reqwest::Error> {
        /*
        Take the cookies from the front end request and add them to a new reqwest client
        that will talk to the back end.
         */
        let mut headers = reqwest::header::HeaderMap::new();
        
        //https://github.com/seanmonstar/reqwest/issues/1636
        /* for cookie in jar.iter() {
            let cookie_name = cookie.name();
            let cookie_value = cookie.value();
            let entry = format!("{}={}", cookie_name, cookie_value);
            println!("\na cookie: {}", entry);
            headers.append(
                reqwest::header::COOKIE,
                entry.parse().unwrap()
            );
            
        } */

        //Hack around a possible reqwest bug
        let mut all_cookies = String::new();
        for cookie in jar.iter() {
            all_cookies = all_cookies + &format!(" {}={};", cookie.name(), cookie.value())[..];
        }
        headers.insert(
            reqwest::header::COOKIE,
            all_cookies.parse().unwrap()
        );


        reqwest::Client::builder()
        .default_headers(headers)
        .cookie_store(true)
        .build()
    }

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
        let mut target_url : reqwest::Url = reqwest::Url::parse(&format!("http://back/users/{}", id)).unwrap();
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
}