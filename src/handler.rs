use actix_web::{Responder, get};

#[get("/hello")]
pub async fn hello_world() -> impl Responder{
    "hello_world!"
}