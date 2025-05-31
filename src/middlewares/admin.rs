use actix_web::{body::MessageBody, dev::{ServiceRequest, ServiceResponse}, middleware::Next, Error, HttpMessage};
use jsonwebtoken::{decode, DecodingKey, Validation};

use crate::{errors::CustomError, schema::JWTClaims};

pub async fn admin_middleware(
    req:ServiceRequest,
    next: Next<impl MessageBody>
) -> Result<ServiceResponse<impl MessageBody>, Error>{

    let auth_header = req.headers().get("Authorization");

    if auth_header.is_none(){
        return  Err(Error::from(CustomError{error:"token missing".to_string()}));
    }

    let token = auth_header.unwrap().to_str();

    if token.is_err(){
        return Err(Error::from(CustomError{error:"Internal Server Error".to_string()}));
    }

    let token = token.unwrap();

    let jwt_secret = std::env::var("ADMIN_JWT_PASSWORD").unwrap();

    let decoded = decode::<JWTClaims>(token,
         &DecodingKey::from_secret(jwt_secret.as_bytes()), &Validation::default());

    if decoded.is_err(){
        println!("error while decoding admin jwt key");
        return Err(Error::from(CustomError{error:"Internal Error".to_string()}));
    }

    let admin_email = decoded.unwrap().claims.sub;

    req.extensions_mut().insert(admin_email);

    next.call(req).await


}