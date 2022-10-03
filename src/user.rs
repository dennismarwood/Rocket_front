use rocket::serde::json::{Value, json};
use rocket::response::{status};
use rocket::response::content::RawHtml;
use rocket::form::Form;
use serde::Serialize;
use rocket::http::{CookieJar, Cookie, SameSite};
use reqwest::{cookie::Jar};
use rocket_dyn_templates::{Template, context};

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
    pub async fn get_user(jar: &CookieJar<'_>) -> Template {//RawHtml<String> {   

        let mut my_url : reqwest::Url = reqwest::Url::parse("http://back/user").unwrap();
        my_url.set_port(Some(8001)).map_err(|_| "cannot be base").unwrap();

        let my_client = reqwest_client(jar).unwrap();

        match my_client.get(my_url).send().await
            {
                Ok(response) => {
                    match response.status() {
                        reqwest::StatusCode::OK => {
                            let user: Value = serde_json::from_str(&response.text().await.unwrap()[..]).unwrap();
                            println!("{:?}", user);
                            Template::render("user", context!{user})
                            //RawHtml(t)
                        }
                        _ => {
                            //RawHtml(response.text().await.unwrap())}
                            Template::render("index", context! {})
                        }
                    }
                },
                Err(e) => Template::render("index", context! {}),//RawHtml(e.to_string()),
            }
    }
}