use std::collections::HashMap;

//https://jsonapi.org/format/
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct JSONAPIError {
    pub status: String, //"401"
    pub canonical: String, //Unauthorized
    pub title: String, //JWT not authorized.
    pub detail: String, //Your session is expired.
}


#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct JSONAPIPatch {
    pub id: String, //1
    pub type_: String, //"articles"
    pub attributes: Option(Vec<HashMap<String, String>>),
    pub relationships: Option(
            Vec<
                (String,//"author"
                    (String,//"data"
                        Vec<
                            HashMap<String, String>//"type": "people"
                        >
                    )
                )
            >
    ), 
}
