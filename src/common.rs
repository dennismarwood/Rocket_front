use rocket::http::{CookieJar};

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
    //https://github.com/seanmonstar/reqwest/issues/1636
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