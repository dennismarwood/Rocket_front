use serde::{Serialize, Deserialize};
use serde::de::DeserializeOwned;
use crate::errors::{ResponseError};
use reqwest;

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
    pub status_code: Option<u16>,
}

impl Response {
    pub async fn new(r: reqwest::Response) -> Result<Self, ResponseError> {
        println!("{:?}", r.text().await);
        todo!();
        let status_code = r.status().as_u16();
        let mut response = r.json::<Self>().await.map_err(|e| ResponseError::DeserializeError(e))?;
        response.status_code = Some(status_code);
        Ok(response)
    }

    pub async fn get_data<T: DeserializeOwned>(r: reqwest::Response) -> Result<T, ResponseError> {
        //Expect a response from the back end to include Some(serde::Value) in the Response.data field.
        
        //Convert response from the back end into a Response struct.
        let resp: Self = Self::new(r).await?;
        //Extract serde Value from the Response.data field.
        let data = match resp.data {
            Some(data) => data,
            None => return Err(ResponseError::NoDataField) //The optional data field is not present.
        };

        //Return a Value or return the serde error.
        let deserialized_data: T = serde_json::from_value(data).map_err(ResponseError::SerdeError)?;

        Ok(deserialized_data)
    }
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

#[derive(Serialize, Deserialize, Default, Clone, Debug)]
pub struct Post {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i32>,

    pub title: Option<String>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
    
    #[serde(skip_serializing_if = "Option::is_none")] 
    pub created: Option<chrono::NaiveDateTime>,
   
    //#[serde(skip_serializing_if = "Option::is_none")]
    pub last_updated: Option<chrono::NaiveDate>,
    
    pub content: Option<String>,
}

#[derive(Serialize, Deserialize, Default, Clone, Debug, FromForm)]
pub struct PostForm {
    pub id: Option<i32>,
    pub title: Option<String>,
    pub author: Option<String>,
    pub created: Option<String>,
    pub last_updated: Option<String>,
    pub content: Option<String>,
    pub tags: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Tag {
    pub id: i32,
    pub name: String,
}

#[derive(Serialize, Deserialize, Default, Clone, Debug)]
pub struct PostAndTag {
    pub post: Post,
    pub tags: Vec<Tag>, 
}