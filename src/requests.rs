use reqwest::Request;
use serde::Serialize;
use serde::de::DeserializeOwned;
use url::{Url};
use crate::models::{Response};
use crate::common::{reqwest_client};
use rocket::http::{CookieJar};
use crate::errors::{BuildUrlError, InitializeRequestError, GetAndProcessError, DecipherResponseError, PatchValueError, PostValueError};

/*
    Requests to the back end.
    The errors are broadly broken into either:
        Failure to create the reqwest::request or receive a reply from the back end -> InitializeRequestError
        An unexpected response code in the reply from the back end -> DecipherResponseError
        Failure to convert the reply into a Response Struct (missing fields, empty fields, failure to serailize, etc) -> ResponseError
        (If applicable) Failure to perform some action specific to the called method -> MethodNameError
*/

pub fn build_url(path: &str) -> Result<Url, BuildUrlError> {
    const TARGET: &'static str = "http://back/api/";
    const PORT: u16 = 8001;

    let base = Url::parse(TARGET).map_err(|e| BuildUrlError::ParseError(e))?;
    let mut url = base.join(path).map_err(|e| BuildUrlError::ParseError(e))?;
    url.set_port(Some(PORT)).map_err(|_| BuildUrlError::BadPortValue(PORT))?;
    
    Ok(url)
}

/**
 Given a path to a backend resource,<br>
 Create and send a get request to backend,<br>
 Return Desirialized value from backend response's "data" field.<br>
 Any error encountered along the way will be returned instead.
 */
pub async fn get_and_process_data<T: DeserializeOwned>(jar: &CookieJar<'_>, path: &str) -> Result<T, GetAndProcessError> {
    //Create a reqwest::client, create a url from the provided path, then perform a get reqwest
    let client = reqwest_client(jar).map_err(|e| InitializeRequestError::BuildRequestClientError(e))?;
    let url = build_url(path).map_err(|e| InitializeRequestError::BuildUrlError(e))?;
    let response = client.get(url).send().await.map_err(|e| InitializeRequestError::RequestResponseError(e))?;
  
    //A 2xx status is expected in this reponse.
    if !response.status().is_success() {
        return Err(DecipherResponseError::UnexpectedResponseCode(response.status()))
            .map_err(GetAndProcessError::DecipherResponseError);
    }
    //Response should contain body. get_data will convert the body to a Response struct, and extract the expected data field.
    let data = Response::get_data(response).await.map_err(|e| GetAndProcessError::ResponseError(e))?;

    //Data field should contain a serde Value item. Now interpret that value.
    let deserialized_data: T = serde_json::from_value(data).map_err(|e| GetAndProcessError::SerdeError(e))?;

    Ok(deserialized_data)
}

/** 
 An Ok response will contain a response for a success or failure post attempt.
 <br>OR<br>
 An Error about intiating or deciphering a response.
 */
pub async fn post_value<T: Serialize>(jar: &CookieJar<'_>, path: &str, body: T) ->  Result<Response, PostValueError> {
    //Create request and url
    let client = reqwest_client(jar).map_err(|e| InitializeRequestError::BuildRequestClientError(e))?;
    let url = build_url(path).map_err(|e| InitializeRequestError::BuildUrlError(e))?;
    //Send request
    let r = client.post(url).json(&body).send().await.map_err(|e| InitializeRequestError::RequestResponseError(e))?;
    Ok(Response::new(r).await?)

}

/** 
 Similar to the post_value function but does not generate a response struct.<br>
 Use this when you expect the back end to return 2xx without a body.
 An Ok response will contain the response code the server returned.
 <br>OR<br>
 An Error about intiating or deciphering a response.
 */
pub async fn post_value_get_status_code<T: Serialize>(jar: &CookieJar<'_>, path: &str, body: T) ->  Result<u16, PostValueError> {
    //Create request and url
    let client = reqwest_client(jar).map_err(|e| InitializeRequestError::BuildRequestClientError(e))?;
    let url = build_url(path).map_err(|e| InitializeRequestError::BuildUrlError(e))?;
    //Send request
    let r = client.post(url).json(&body).send().await.map_err(|e| InitializeRequestError::RequestResponseError(e))?;
    Ok(r.status().as_u16())
}

/** 
A <b>successful</b> patch will return Ok(None).<br>
Any non 2xx reply will attempt to return Ok(Response).
*/
pub async fn patch_value<T: Serialize>(jar: &CookieJar<'_>, path: &str, body: T) ->  Result<Option<Response>, PatchValueError> {
    //The back end should return a 204 and an empty body in a successful update.
    //A successfull operation will retun Ok(None)
    //If the back end returns a non 2xx response, return an Ok(Response)
    //Any unexpected failures along the way return a PatchValueError

    //Create request and url
    let client = reqwest_client(jar).map_err(|e| InitializeRequestError::BuildRequestClientError(e))?;
    let url = build_url(path).map_err(|e| InitializeRequestError::BuildUrlError(e))?;
    //Send request
    let r = client.patch(url).json(&body).send().await.map_err(|e| InitializeRequestError::RequestResponseError(e))?;

    //Expecting no body and a 204 status code
    if !r.status().is_success() {
        //A body is expected in a non 2xx response.
        println!("FAIL");
        println!("{}", r.status());
        return Ok(Some(Response::new(r).await?));
    }

    //A successfull patch has no body.
    Ok(None)
}