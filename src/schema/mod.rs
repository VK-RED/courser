use serde::{Deserialize, Serialize};

pub mod user;
pub mod admin;

#[derive(Deserialize, Serialize, Debug)]
pub struct JWTClaims{
    pub sub: String,
    pub exp: usize,
}
pub struct StructWithId{
    pub id: String,
}

pub struct StructWithVal{
    pub val: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SignupResponse{
    pub message: String,
    pub id: String
}

#[derive(Deserialize, Serialize)]
pub struct EmailAndPassword{
    pub email: String,
    pub password: String,
}

#[derive(Serialize, Deserialize)]
pub struct SigninResponse{
    pub message: String,
    pub token: String,
}

#[derive(Serialize, Clone)]
pub struct StructWithEmail{
    pub email: String,
}

#[derive(Serialize)]
pub struct PurchaseResponse{
    pub id: String,
    pub message: &'static str,
}