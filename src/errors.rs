use thiserror::Error;
use url::ParseError;
use serde::{Serialize, Deserialize};

#[derive(Error, Debug)]
pub enum BuildRequestClientError {
    #[error(transparent)]
    ReqwestError(#[from] reqwest::Error),
}

#[derive(Error, Debug)]
pub enum BuildUrlError {
    #[error("Setting port value of '{0}' caused the Url to have an invalid base.")]
    BadPortValue(u16),
    #[error(transparent)]
    ParseError(#[from] ParseError),
}

#[derive(Error, Debug)]
pub enum RequestResponseError {
    //The back end did not respond or returned an invalid response.
    #[error(transparent)]
    ReqwestError(#[from] reqwest::Error),
}

#[derive(Error, Debug)]
pub enum InitializeRequestError {
    //All requests build a client, build a url, and use the client to send a request to the back end.
    #[error(transparent)]
    BuildRequestClientError(#[from] BuildRequestClientError),
    #[error(transparent)]
    BuildUrlError(#[from] BuildUrlError),
    #[error(transparent)]
    RequestResponseError(#[from] reqwest::Error),
}

#[derive(Error, Debug)]
pub enum DecipherResponseError {
    #[error("Received unexpected response status code: {0}")]
    UnexpectedResponseCode(reqwest::StatusCode),
}

#[derive(Error, Debug)]
pub enum ResponseError {
    //Errors for Response struct impls.
    #[error(transparent)]
    DeserializeError(#[from] reqwest::Error),
    #[error(transparent)]
    SerdeError(#[from] serde_json::Error),
    #[error("The response from the back end did not include the expected data field.")]
    NoDataField,
}

#[derive(Error, Debug)]
pub enum GetAndProcessError {
    #[error(transparent)]
    InitiateRequestError(#[from] InitializeRequestError),
    #[error(transparent)]
    DecipherResponseError(#[from] DecipherResponseError),
    #[error(transparent)]
    ResponseError(#[from] ResponseError),
    #[error(transparent)]
    SerdeError(#[from] serde_json::Error),
}


#[derive(Error, Debug)]
pub enum PatchValueError {
    #[error(transparent)]
    InitiateRequestError(#[from] InitializeRequestError),
    #[error(transparent)]
    ResponseError(#[from] ResponseError),
}

#[derive(Error, Debug)]
pub enum PostValueError {
    #[error(transparent)]
    InitiateRequestError(#[from] InitializeRequestError),
    #[error(transparent)]
    ResponseError(#[from] ResponseError),
}