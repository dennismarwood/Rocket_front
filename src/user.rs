use rocket::serde::json::{Value, json};
use rocket::response::{status, Redirect, Flash};
use rocket::response::content::RawHtml;
use rocket::form::Form;
use serde::Serialize;
use rocket::http::{CookieJar, Cookie, SameSite, Status};
use reqwest::{cookie::Jar};
use rocket_dyn_templates::{Template, context};
use rocket::request::{local_cache, FlashMessage};
use rocket::Request;


#[derive(FromForm, Serialize)]
pub struct User {
    pub email : Option<String>,
    pub pass :Option<String>, // Must be labled phc for diesel but this will initially be the user's new password
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub role: Option<i32>,
    pub active: Option<bool>,
}

pub mod routes {
    use super::*;

    /*
    User has jwt cookie? No -> redirect to login page.
    Send jwt to back end. Receive user fields.
    Display user fields form. Post to update user.
    */

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

    #[get("/")]
    pub async fn get_user(jar: &CookieJar<'_>) -> Result<Template, Flash<Redirect>> {//RawHtml<String> {   

        let mut my_url : reqwest::Url = reqwest::Url::parse("http://back/user").unwrap();
        my_url.set_port(Some(8001)).map_err(|_| "cannot be base").unwrap();

        let my_client = reqwest_client(jar).unwrap();

        //let mut map = serde_json::Map::new();

        match my_client.get(my_url).send().await
            {
                Ok(response) => {
                    match response.status() {
                        reqwest::StatusCode::OK => {
                            let user: Value = serde_json::from_str(&response.text().await.unwrap()[..]).unwrap();
                            Ok(Template::render("user", context!{user}))
                        }
                        reqwest::StatusCode::INTERNAL_SERVER_ERROR => { //This is when the back end returns a 500
                            let response: Value = serde_json::from_str(&response.text().await.unwrap()[..]).unwrap();
                            Ok(Template::render("error/500", context! {response}))
                        }
                        reqwest::StatusCode::UNAUTHORIZED => {
                            //Err(Redirect::to(uri!("/login")))
                            Err(Flash::error(Redirect::to("/login"), "You must be logged in to view your user profile."))
                        }
                        _ => { //Backend returned status code other than 500, 401
                            let response: Value = serde_json::from_str(&response.text().await.unwrap()[..]).unwrap();
                            Ok(Template::render("error/default", context! {response}))
                            
                        }
                    }
                },
                Err(e) => { //This is when the front end returns a 500
                    let response = e.to_string();
                    Ok(Template::render("error/500", context! {response}))
                }
            }
    }
}