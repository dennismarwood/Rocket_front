//use rocket::Request;
//use rocket::http::Status;
//use rocket_dyn_templates::{Template, context};


//use super::*;

// #[catch(default)]
// pub fn default_catcher(status: Status, request: &Request) -> Template { 
//     //println!("\n\n{:?}", status);
//     //println!("\n\n{:?}", request);
//     println!("\ndefault_catcher ran.");
//     Template::render("error/default", context! {})
// }

// #[catch(500)]
// pub fn five_hundread(status: Status, request: &Request) -> Template {
//     //println!("\n\n{:?}", status);
//     //println!("\n\n{:?}", request);
//     println!("\nfive_hundread catcher ran.");
//     Template::render("error/default", context! {})
// }

#[catch(404)]
pub fn not_found() -> &'static str {
    "Gen 404"
}

#[catch(404)]
pub fn post_not_found() -> &'static str {
    "POST Gen 404"
}