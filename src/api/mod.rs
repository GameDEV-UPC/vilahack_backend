use axum::http::StatusCode;
use axum_extra::TypedHeader;
use headers::{Authorization, authorization::Bearer};

use crate::authentication::authenticate;

pub async fn test(
    TypedHeader(Authorization(bearer)): TypedHeader<Authorization<Bearer>>,
) -> Result<String, StatusCode> {
    match authenticate(bearer.token()) {
        Err(err) => eprintln!("err: {err:?}"),
        Ok(val) => println!("val: {val}"),
    }

    Ok(String::from("tested"))
}
