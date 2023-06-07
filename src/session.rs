use rocket::serde::json::{Value};
use rocket::response::Redirect;
//use rocket::response::content::RawHtml;
use rocket::form::Form;
use serde::Serialize;
use rocket::http::{Cookie, CookieJar, SameSite};
use rocket::request::FlashMessage;
use rocket_dyn_templates::{Template, context};

#[derive(FromForm, Serialize)]
pub struct LoginCredentials {
    pub email: String,
    pub password: String,
}

#[get("/login")]
pub async fn login(flash: Option<FlashMessage<'_>>) -> Template {
    Template::render("login", context!{flash})
}
 
#[post("/", data = "<user_input>")]
pub async fn process_login(user_input: Form<LoginCredentials>, jar: &CookieJar<'_>) -> Result<Redirect, Value> {
    let mut my_url : reqwest::Url = reqwest::Url::parse("http://back/users/session").unwrap();
    my_url.set_port(Some(8001)).map_err(|_| "cannot be base").unwrap();

    //let client = reqwest::Client::new();
    let client = reqwest::Client::builder()
        .cookie_store(true)
        .build().unwrap();
    //Send the user input to the backend.
    match client.post(my_url)
        .json(&user_input.into_inner())
        .send()
        .await
    {
        Ok(back_end_response) => {
            match back_end_response.status() {
                reqwest::StatusCode::OK => {
                    //A jwt should be sitting in the cookie jar
                    let mut jwt = String::from("");
                    for cookie in back_end_response.cookies(){
                        if cookie.name() == "jwt" {
                            jwt = String::from(cookie.value());
                        }
                    }
                    if jwt.is_empty() {
                        //return Status::InternalServerError("Expected backend to return a jwt. It did not.")
                        return Err(serde_json::from_str("Expected backend to return a jwt. It did not.").unwrap())
                    }
                    //Create the jwt cookie for the front end.
                    let new_jwt = Cookie::build(String::from("jwt"), String::from(jwt))
                    .http_only(true)
                    .path("/")
                    .same_site(SameSite::Strict);
                    //Add this cookie to the front end's response.
                    jar.add(new_jwt.finish());
                    //return RawHtml(String::from("Added jwt."))
                    Ok(Redirect::to(uri!("/user")))
                },
                reqwest::StatusCode::UNAUTHORIZED => {
                    Err(serde_json::from_str(back_end_response.text().await.unwrap().as_str()).unwrap())
                },
                _ =>Err(serde_json::from_str(back_end_response.text().await.unwrap().as_str()).unwrap()),
            }
        }
        Err(e) => Err(serde_json::from_str(e.to_string().as_str()).unwrap()), //boom
    }
}

#[get("/logout")]
pub async fn logout(jar: &CookieJar<'_>) -> Redirect {
    jar.remove(Cookie::named("jwt"));
    Redirect::to(uri!("/"))
}