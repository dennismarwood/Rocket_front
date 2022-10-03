use rocket::Request;
use rocket::http::Status;
use rocket_dyn_templates::{Template, context};

pub mod routes {
    use super::*;

    #[catch(default)]
    pub fn default_catcher(status: Status, request: &Request) -> Template { 
        println!("\n\n{:?}", status);
        println!("\n\n{:?}", request);
        Template::render("error/default", context! {})
    }

    #[catch(500)]
    pub fn five_hundread(status: Status, request: &Request) -> Template { 
        println!("\n\n{:?}", status);
        println!("\n\n{:?}", request);
        Template::render("error/default", context! {})
    }
}