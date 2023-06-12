use thiserror::Error;
use url::ParseError;

#[derive(Error, Debug)]
pub enum ResponseError {
    #[error("The response from the back end did not include the expected data field.")]
    NoDataField,
    #[error(transparent)]
    SerdeError(#[from] serde_json::Error),
    #[error(transparent)]
    ReqwestError(#[from] reqwest::Error),
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

#[derive(Error, Debug)]
pub enum BuildUrlError {
    #[error("Setting port value of '{0}' caused the Url to have an invalid base.")]
    BadPortValue(u16),
    #[error(transparent)]
    ParseError(#[from] ParseError),
}