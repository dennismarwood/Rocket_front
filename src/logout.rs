use rocket::http::{Cookie, CookieJar};
use rocket::response::Redirect;

pub mod routes {
    use super::*;

    #[get("/")]
    pub async fn logout(jar: &CookieJar<'_>) -> Redirect {
        jar.remove(Cookie::named("jwt"));
        Redirect::to(uri!("/"))
    }
}