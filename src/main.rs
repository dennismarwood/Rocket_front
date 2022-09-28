#[macro_use] extern crate rocket;
extern crate reqwest;

mod login;

mod user;
use user::routes::*;

#[get("/")] //This is a macro attribute
fn index() -> &'static str {
    "Dennis Marwood\n"
}

#[get("/blog")]
pub async fn blog() -> Result<String, String> {
    //match reqwest::get("http://localhost:8001/api").await//.unwrap().text().await;
    let mut my_url : reqwest::Url = reqwest::Url::parse("http://back").unwrap();
    my_url.set_port(Some(8001)).map_err(|_| "cannot be base").unwrap();
    match reqwest::get(my_url).await
    {
        Ok(x) => Ok(format!("{}: {:?}", x.status() , x.text().await)),
        Err(e) => Err(e.to_string()),
    }
    //let x = json_response_from_backend.unwrap();
    //String::from("Blog")
}
/* 
#[catch(404)]
fn not_found() -> Value {
    json!("Not Found!")
}

#[catch(401)]
fn unauthorized() -> Value {
    json!("Unauthorized")
}

#[catch(422)]
fn unprocessable_entity() -> Value {
    json!("422 - Unprocessable Entity. Verify that submitted data is valid.")
}
 */

#[launch] //Will genererate the main function
fn rocket() -> _ { //Built the rocket here
    rocket::build()
        .mount("/", routes![index, blog])
        .mount("/user", routes![get_user])
        .mount("/login", routes![login::routes::login, login::routes::process_login])
        .mount("/forwards_example", routes![])
        //.register("/udemy/2/5", catchers![not_found, unauthorized, unprocessable_entity])
}