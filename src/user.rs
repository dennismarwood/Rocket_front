use rocket::serde::json::{Value, json};
use rocket::Request;
use rocket::response::{Redirect, Flash, status};
use serde::Serialize;
use rocket::http::{CookieJar, Status};
use rocket_dyn_templates::{Template, context};
use rocket::form::Form;
/*
{"data":
    {"active":true,
    "created":"2022-08-29T01:32:11",
    "email":"dennismarwood@gmail.com",
    "first_name":"Dennis",
    "id":1,
    "last_access":"2023-05-20",
    "last_name":"Marwood",
    "role":1
},"status":"Success"}
*/
//#[derive(FromForm, Serialize)]
//pub struct 200_Response
#[derive(FromForm, Serialize)]
pub struct Userx {
    pub email : Option<String>,
    pub pass :Option<String>, // Must be labled phc for diesel but this will initially be the user's new password
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub role: Option<i32>,
    pub active: Option<bool>,
}

#[derive(FromForm, Serialize)]
pub struct UserUpdates {
    pub email: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
}

#[derive(FromForm, Serialize)]
pub struct PasswordUpdate {
    pub current_password: String,
    pub new_password: String,
    pub new_password_confirm: String,
}

pub mod routes {
    use reqwest::header::CONTENT_TYPE;

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

    #[get("/update_pw")]
    pub async fn update_pw_template()-> Template {
        Template::render("update_pw", context!{})
    }

    #[post("/process_pw_update", data = "<pw_update>")]
    pub async fn process_pw_update(pw_update: Form<PasswordUpdate>, jar: &CookieJar<'_>) {
        println!("Here is pw_update {}", pw_update.current_password);
        let mut target_url : reqwest::Url = reqwest::Url::parse("http://back/users/confirm_pw").unwrap();
        target_url.set_port(Some(8001)).map_err(|_| "cannot be base").unwrap();

        let my_client = reqwest_client(jar).unwrap();

        match my_client.post(target_url).header(CONTENT_TYPE, "text/plain").body(pw_update.current_password.clone()).send().await{
            //Need to look at the response code type and handle pw validity
            Ok(response) => println!("PW was valid\n{:?}", response.text().await),
            Err(_) => println!("PW was NOT valid")
        }
        //verify that user password was valid, logout if not.
        //verify that the new passwords match, return to form with flash error if not.
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
                        let user: Value = serde_json::from_str(&response.text().await.unwrap()[..]).unwrap();
                        let user = &user["data"];
                        //let user = response.json().await().unwrap();
                        Ok(Template::render("user", context!{user}))
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
    pub async fn patch_user(user_input: Form<UserUpdates>, jar: &CookieJar<'_>) -> Result<Template, Flash<Redirect>> {
        let mut target_url : reqwest::Url = reqwest::Url::parse("http://back/user").unwrap();
        target_url.set_port(Some(8001)).map_err(|_| "cannot be base").unwrap();

        let my_client = reqwest_client(jar).unwrap();

        match my_client.patch(target_url)
            //.form(&user_input.into_inner())
            .json(&user_input.into_inner())
            .send()
            .await{
            Ok(response) => {
                match response.status() {
                    reqwest::StatusCode::OK => {
                        let user: Value = serde_json::from_str(&response.text().await.unwrap()[..]).unwrap();
                        Ok(Template::render("user", context!{user}))
                    }
                    reqwest::StatusCode::INTERNAL_SERVER_ERROR => { //Backend returns a 500
                        let response: Value = serde_json::from_str(&response.text().await.unwrap()[..]).unwrap();
                        Ok(Template::render("error/500", context! {response}))
                    }
                    reqwest::StatusCode::UNAUTHORIZED => {
                        Err(Flash::error(Redirect::to("/login"), "You must be logged in to view your user profile."))
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

}