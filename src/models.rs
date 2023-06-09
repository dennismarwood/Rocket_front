use serde::{Serialize, Deserialize};

#[derive(FromForm, Serialize, Deserialize, Debug)]
pub struct User {
    pub email : Option<String>,
    pub pass :Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub role: Option<i32>,
    pub active: Option<bool>,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct UserWithoutPHC {
    pub id: i32,
    pub email: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub created: Option<String>,
    pub role: i32,
    pub active: Option<bool>,
    pub last_access: Option<String>,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct Response {
    pub status: String,
    pub message: Option<String>,
    pub location: Option<String>,
    pub data: Option<serde_json::Value>,
    pub code: Option<String>,
    pub errors: Option<serde_json::Value>,
}

#[derive(FromForm, Serialize, Deserialize)]
pub struct UserUpdates {
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub email: Option<String>,
    pub phc: Option<String>,
    pub active: Option<bool>,
}

#[derive(FromForm, Serialize, Debug)]
pub struct PasswordUpdate<'r> {
    pub current_password:  &'r str,
    pub new_password: &'r str,
    #[field(validate = eq(self.new_password))]
    pub new_password_confirm:  &'r str,
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct Post {
    pub id: Option<i32>,
    pub title: Option<String>,
    pub author: Option<String>,
    pub created: Option<chrono::NaiveDateTime>,
    pub last_updated: Option<chrono::NaiveDate>,
    pub content: Option<String>,
    pub tag: Option<Vec<Tag>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Tag {
    pub id: i32,
    pub name: String,
}
