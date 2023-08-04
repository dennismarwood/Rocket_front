#[macro_use] extern crate rocket;
extern crate rocket_dyn_templates;
use rocket_dyn_templates::{Template, context};
extern crate reqwest;
extern crate tera;

extern crate url;

mod session;
mod models;

mod user;
use user::*;

mod post;

mod tag;
use tag::*;

mod catchers;
use catchers::{not_found, post_not_found};

mod common;

mod requests;

mod errors;

mod tera_custom;
use tera_custom::*;

#[get("/")]
fn landing_page() -> Result<Template, String > {
    /*
        Info about me. Soft resume or cover letter. Some links to blog posts I want to highlight. Maybe some posts based on date.
     */
    Ok(Template::render("post", context!{}))
}



#[get("/blog/<first_name>")]
pub async fn blog(first_name: String) -> Result<String, String> {
    /*
        A user's blog landing page. List of posts, tag cloud, etc. A kind of homepage for the user. Info about them.
     */
    //match reqwest::get("http://localhost:8001/api").await//.unwrap().text().await;
    let mut my_url : reqwest::Url = reqwest::Url::parse("http://back").unwrap();
    my_url.set_port(Some(8001)).map_err(|_| "cannot be base").unwrap();
    match reqwest::get(my_url).await
    {
        Ok(x) => Ok(format!("{}: {:?}", x.status() , x.text().await)),
        Err(e) => Err(e.to_string()),
    }
}

#[launch] //Will genererate the main function
fn rocket() -> _ { //Built the rocket here
    rocket::build()
        .mount("/", routes![landing_page, blog])
        .mount("/user", routes![get_user, patch_user, update_pw_template, process_pw_update, list_all_users, get_user_by_id, patch_user_by_id, delete_user, existing_post, new_post, process_post])
        //.register("/user", catchers![user_401])
        .mount("/tag", routes![get_tags, add_tag])
        .mount("/session", routes![session::login, session::process_login, session::logout])
        .mount("/post", routes![post::new_post, post::existing_post])
        .mount("/forwards_example", routes![])
        .attach(Template::custom(|engine| {
            engine.tera.register_tester("my_odd", testers::my_odd())
        }))
        .register("/", catchers![not_found])
        .register("/post", catchers![post_not_found])
        //.register("/udemy/2/5", catchers![not_found, unauthorized, unprocessable_entity])
}