use std::str::FromStr;

use actix_web::{get, post, web::{self, Json}, HttpMessage, HttpRequest, HttpResponse, Responder};
use chrono::{Utc, Duration};
use jsonwebtoken::{encode, EncodingKey, Header};
use sqlx::types::{Uuid};
use crate::{errors::{CustomError}, models::{purchase::get_user_purchases, user::{check_user_exists, create_user, get_user_id_by_email, retrieve_password}}, schema::{user::CreateUser, EmailAndPassword, JWTClaims, SigninResponse, SignupResponse, StructWithEmail}, utils::{hash_password, verify_password}, GlobalState};

#[post("/signup")]
async fn signup_user(data:web::Data<GlobalState>, user:Json<CreateUser>) -> impl Responder{
    let user_exists = check_user_exists(&data.pool, &user.email).await;

    if let Err(e) = user_exists{
        return HttpResponse::InternalServerError().json(e);
    }

    if user_exists.unwrap() == true{
        return HttpResponse::BadRequest().json(CustomError{error:"User exists already with this email".to_string()});
    }

    let password_hash = hash_password(&user.password);

    if let Err(_e) = password_hash{
        return HttpResponse::InternalServerError().json(CustomError{error:"Something went wrong !".to_string()});
    }

    let user_meta = CreateUser{
        email: user.email.clone(),
        name: user.name.clone(),
        password: password_hash.unwrap()
    };

    let signup_result = create_user(&data.pool, user_meta).await;

    match signup_result {
        Ok(res) => HttpResponse::Ok().json(SignupResponse{message:String::from("Signed up successfully"),id: res}),
        Err(e) => HttpResponse::BadGateway().json(e),
    }
}

#[post("/signin")]
async fn signin_user(data:web::Data<GlobalState>, user_data:web::Json<EmailAndPassword>) -> impl Responder {

    let user_exists = check_user_exists(&data.pool, &user_data.email).await;

    if let Err(e) = user_exists{
        return HttpResponse::InternalServerError().json(e);
    }

    // throw when user not found
    if user_exists.unwrap() == false {
        return HttpResponse::BadRequest().json(CustomError{error:"Signup first".to_string()});
    }

    let pool = &data.pool;
    let password_hash_res = retrieve_password(pool, &user_data.email).await;

    if let Err(e) = password_hash_res{
        return HttpResponse::InternalServerError().json(e);
    }

    let hash = password_hash_res.unwrap();
    let is_valid = verify_password(&user_data.password, &hash);

    let jwt_secret = std::env::var("USER_JWT_PASSWORD").unwrap();

    let tomorrow = Utc::now() + Duration::days(1);

    let claims = JWTClaims{
        sub: user_data.email.clone(),
        exp: tomorrow.timestamp() as usize
    };

    let token = encode(&Header::default(), &claims, &EncodingKey::from_secret(jwt_secret.as_bytes()));

    if let Err(_) = token {
        return HttpResponse::InternalServerError().json(CustomError{error:"Internal Error".to_string()});
    }

    match is_valid {
        Ok(()) => HttpResponse::Ok().json(SigninResponse{message:String::from("Signined in Successfully".to_string()), token:token.unwrap()}),
        Err(_) => HttpResponse::BadRequest().json(CustomError{error:"Enter Valid Password".to_string()}) 
    }

}

#[get("")]
async fn user_purchases(data:web::Data<GlobalState>, req:HttpRequest) -> impl Responder{
    let user_struct = req.extensions().get::<StructWithEmail>().cloned();

    if user_struct.is_none(){
        return HttpResponse::Forbidden().json(CustomError{error:"email missing".to_string()});
    }

    let user_email = user_struct.unwrap().email;

    let pool = &data.pool;

    let user_id_res = get_user_id_by_email(pool, &user_email).await;

    if let Err(e) = user_id_res{
        return HttpResponse::Forbidden().json(e);
    }

    let user_id = user_id_res.unwrap();

    let user_uuid_res = Uuid::from_str(&user_id);

    if user_uuid_res.is_err(){
        return HttpResponse::Forbidden().json(CustomError{error:"Internal Error".to_string()})
    }

    let purchases_res = get_user_purchases(pool, user_uuid_res.unwrap()).await;

    match purchases_res {
        Ok(purchases) => HttpResponse::Ok().json(purchases),
        Err(e) => HttpResponse::BadRequest().json(e)
    }
    

}

#[cfg(test)]
mod tests{

    use crate::{models::purchase::Purchase, test_init_app::init};
    use actix_web::test;
    use super::*;

    #[actix_web::test]
    async fn test_signup_and_signin(){
        let (app, pool) = init(signup_user).await;

        let user = CreateUser{
            email: String::from("vk@gmail.com"),
            name: String::from("Iron Man"),
            password: String::from("THERIYATHU")
        };

        let res = test::TestRequest::post()
        .set_json(user)
        .uri("/api/v1/user/signup")
        .send_request(&app)
        .await;

        assert!(res.status().is_success());

        let signup_res_body:SignupResponse = test::read_body_json(res).await;
        
        assert_eq!(signup_res_body.message, "Signed up successfully".to_string());

        let json = EmailAndPassword {
            email: "vk@gmail.com".to_string(),
            password: "THERIYATHU".to_string(),
        };

        let res = test::TestRequest::post()
        .set_json(json)
        .uri("/api/v1/user/signin")
        .send_request(&app)
        .await;

        assert!(res.status().is_success());

        let res_body:SigninResponse = test::read_body_json(res).await;

        assert_eq!(&res_body.message, "Signined in Successfully");

        sqlx::query("DELETE FROM user_table WHERE email = $1")
            .bind("vk@gmail.com")
            .execute(&pool)
            .await
            .unwrap();

    }
    
    #[actix_web::test]
    async fn test_invalid_credentials(){
        let (app, pool) = init(signup_user).await;

        let user = CreateUser{
            email: String::from("newone@gmail.com"),
            name: String::from("Iron Man"),
            password: String::from("THERIYATHU")
        };

        let res = test::TestRequest::post()
        .set_json(user)
        .uri("/api/v1/user/signup")
        .send_request(&app)
        .await;

        assert!(res.status().is_success());

        let signup_res_body:SignupResponse = test::read_body_json(res).await;
        
        assert_eq!(signup_res_body.message, "Signed up successfully".to_string());

        let json = EmailAndPassword {
            email: "newone@gmail.com".to_string(),
            password: "IRONMAN".to_string(),
        };

        let res = test::TestRequest::post()
        .set_json(json)
        .uri("/api/v1/user/signin")
        .send_request(&app)
        .await;

        let res_body:CustomError = test::read_body_json(res).await;
        assert_eq!(res_body.error, "Enter Valid Password".to_string());

        sqlx::query("DELETE FROM user_table WHERE email = $1")
            .bind("newone@gmail.com")
            .execute(&pool)
            .await
            .unwrap();

    }   

    #[actix_web::test]
    async fn test_signin_with_unused_email(){
        let (app, _pool) = init(signin_user).await;
        let json = EmailAndPassword {
            email: "unused@gmail.com".to_string(),
            password: "THERIYATHU".to_string(),
        };

        let res = test::TestRequest::post()
        .set_json(json)
        .uri("/api/v1/user/signin")
        .send_request(&app)
        .await;

        let res_body:CustomError = test::read_body_json(res).await;
        assert_eq!(res_body.error, "Signup first".to_string());
    }

    #[actix_web::test]
    async fn test_signup_twice(){

        let (app, pool) = init(signup_user).await;

        let user = CreateUser{
            email: String::from("twice@gmail.com"),
            name: String::from("Iron Man"),
            password: String::from("THERIYATHU")
        };

        let res = test::TestRequest::post()
        .set_json(user)
        .uri("/api/v1/user/signup")
        .send_request(&app)
        .await;

        assert!(res.status().is_success());

        let signup_res_body:SignupResponse = test::read_body_json(res).await;
        
        assert_eq!(signup_res_body.message, "Signed up successfully".to_string());

        let user = CreateUser{
            email: String::from("twice@gmail.com"),
            name: String::from("Iron Man"),
            password: String::from("THERIYATHU")
        };

        let res = test::TestRequest::post()
        .set_json(user)
        .uri("/api/v1/user/signup")
        .send_request(&app)
        .await;

        assert!(!res.status().is_success());

        let res_body:CustomError = test::read_body_json(res).await;
        assert_eq!(res_body.error, "User exists already with this email".to_string());

        sqlx::query("DELETE FROM user_table WHERE email = $1")
            .bind("twice@gmail.com")
            .execute(&pool)
            .await
            .unwrap();

    }

    #[actix_web::test]
    async fn test_user_purchases(){
        let (app, pool) = init(signup_user).await;

        let user = CreateUser{
            email: String::from("purchase@test.com"),
            name: String::from("Iron Man"),
            password: String::from("THERIYATHU")
        };

        let res = test::TestRequest::post()
        .set_json(user)
        .uri("/api/v1/user/signup")
        .send_request(&app)
        .await;

        assert!(res.status().is_success());

        let json = EmailAndPassword {
            email: "purchase@test.com".to_string(),
            password: "THERIYATHU".to_string(),
        };

        let res_1 = test::TestRequest::post()
        .set_json(json)
        .uri("/api/v1/user/signin")
        .send_request(&app)
        .await;

        assert!(res_1.status().is_success());

        let res_1_body:SigninResponse = test::read_body_json(res_1).await;
        let token = res_1_body.token;

        let res = test::TestRequest::get()
        .uri("/api/v1/user/purchases")
        .append_header(("Authorization", token))
        .send_request(&app)
        .await;

        assert!(res.status().is_success());

        let res_body:Vec<Purchase> = test::read_body_json(res).await;
        println!("User Purchases: {:?}", res_body);

        sqlx::query("DELETE FROM user_table WHERE email = $1")
            .bind("purchase@test.com")
            .execute(&pool)
            .await
            .unwrap();
    }

    #[actix_web::test]
    async fn test_user_purchases_wo_headers() {
        let (app,_pool) = init(user_purchases).await;
    
        // Method 1: Use call_service instead of send_request
        let req = test::TestRequest::get()
            .uri("/api/v1/user/purchases")
            .to_request();
            
        let res = test::try_call_service(&app, req).await;

        if let Err(e) = res {
            let a: Option<&CustomError> = e.as_error();
            assert!(a.is_some());
            assert_eq!(a.unwrap().error, "Token Not found");
            return;
        }

        panic!("Expected error");

    }


}