use actix_web::{body::MessageBody, dev::{ServiceRequest, ServiceResponse}, middleware::Next, Error, HttpMessage, HttpResponse};
use jsonwebtoken::{decode, DecodingKey, Validation};

use crate::{errors::{AppError, CustomError}, schema::{JWTClaims, StructWithEmail}};

pub async fn user_middleware(
    req:ServiceRequest, 
    next: Next<impl MessageBody>) -> Result<ServiceResponse<impl MessageBody>, Error>
{   

    let authorization = req.headers().get("Authorization");

    if authorization.is_none(){
        return Err(Error::from(CustomError{error:"Token Not found"}));
    }

    let token = authorization.unwrap().to_str();

    if token.is_err() {
        return Err(Error::from(AppError::InternalError));
    }

    let key = std::env::var("USER_JWT_PASSWORD").unwrap();

    let token = token.unwrap();

    let decoded = decode::<JWTClaims>(token, &DecodingKey::from_secret(key.as_bytes()), &Validation::default());

    if decoded.is_err(){
        return Err(Error::from(CustomError{error:"Invalid token"}));
    }

    let claims = decoded.unwrap();

    // add the decoded email to req extensions
    req.extensions_mut().insert(StructWithEmail{email:claims.claims.sub});
    next.call(req).await

}


