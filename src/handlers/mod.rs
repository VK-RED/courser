pub mod admin;
pub mod user;
pub mod course;

use actix_web::{Responder, get};

#[get("/hello")]
pub async fn hello_world() -> impl Responder{
    "hello_world!"
}

#[cfg(test)]
mod tests{
    use actix_web::{test::{self, TestRequest}};

    use super::*;

    #[actix_web::test]
    async fn test_hello_world(){
        let app = crate::test_init_app::init(hello_world).await;

        let req = TestRequest::get().uri("/api/v1/hello").to_request();
        let res = test::call_service(&app, req).await;

        let body_bytes = test::read_body(res).await;
        let body_str = std::str::from_utf8(&body_bytes).unwrap();
        
        assert_eq!(body_str, "hello_world!");
    }
}