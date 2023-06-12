use serde::de::DeserializeOwned;
use url::{Url, ParseError};
use crate::models::{Response};
use crate::common::{reqwest_client};
use rocket::http::{CookieJar};
use thiserror::Error;
use crate::models::{ResponseError};

/*
    Requests to the back end should be fired from this module.
*/

#[derive(Error, Debug)]
pub enum BuildUrlError {
    #[error("Setting port value of '{0}' caused the Url to have an invalid base.")]
    BadPortValue(u16),
    #[error(transparent)]
    ParseError(#[from] ParseError),
}

pub fn build_url(path: &str) -> Result<Url, BuildUrlError> {
    const TARGET: &'static str = "http://back/api/";
    const PORT: u16 = 8001;

    let base = Url::parse(TARGET).map_err(|e| BuildUrlError::ParseError(e))?;
    let mut url = base.join(path).map_err(|e| BuildUrlError::ParseError(e))?;
    url.set_port(Some(PORT)).map_err(|_| BuildUrlError::BadPortValue(PORT))?;
    
    Ok(url)
}

#[derive(Error, Debug)]
pub enum ProcessError {
    #[error("Received bad response: {0}")]
    BadResponse(reqwest::StatusCode),
    #[error(transparent)]
    BuildUrlError(#[from] BuildUrlError),
    #[error(transparent)]
    ResponseError(#[from] ResponseError),
    #[error(transparent)]
    SerdeError(#[from] serde_json::Error),
    #[error(transparent)]
    ReqwestError(#[from] reqwest::Error),
}

pub async fn get_and_process_data<T: DeserializeOwned>(jar: &CookieJar<'_>, path: &str) -> Result<T, ProcessError> {
    //Create a reqwest::client, create a url from the provided path, then perform a get reqwest and return the expected data
    let client = reqwest_client(jar).map_err(|e| ProcessError::ReqwestError(e))?;

    let url = build_url(path).map_err(|e| ProcessError::BuildUrlError(e))?;

    //Send request to back end
    let response = client.get(url).send().await.map_err(|e| ResponseError::ReqwestError(e))?;

    //Error unless recieved response
    if !response.status().is_success() {
        return Err(ProcessError::BadResponse(response.status()));
    }

    let data = Response::get_data(response).await.map_err(|e| ProcessError::ResponseError(e))?;

    //Data field should contain a serde Value item. Now interpret that value.
    let deserialized_data: T = serde_json::from_value(data).map_err(|e| ProcessError::SerdeError(e))?;

    Ok(deserialized_data)
}