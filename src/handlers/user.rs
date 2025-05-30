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
        return HttpResponse::BadRequest().json(CustomError{error:"User exists already with this email"});
    }

    let password_hash = hash_password(&user.password);

    if let Err(_e) = password_hash{
        return HttpResponse::InternalServerError().json(CustomError{error:"Something went wrong !"});
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
        return HttpResponse::BadRequest().json(CustomError{error:"Signup first"});
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
        return HttpResponse::InternalServerError().json(CustomError{error:"Internal Error"});
    }

    match is_valid {
        Ok(()) => HttpResponse::Ok().json(SigninResponse{message:String::from("Signined in Successfully"), token:token.unwrap()}),
        Err(_) => HttpResponse::BadRequest().json(CustomError{error:"Enter Valid Password"}) 
    }

}

#[get("")]
async fn user_purchases(data:web::Data<GlobalState>, req:HttpRequest) -> impl Responder{
    let user_struct = req.extensions().get::<StructWithEmail>().cloned();

    if user_struct.is_none(){
        return HttpResponse::Forbidden().json(CustomError{error:"email missing"});
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
        return HttpResponse::Forbidden().json(CustomError{error:"Internal Error"})
    }

    let purchases_res = get_user_purchases(pool, user_uuid_res.unwrap()).await;

    match purchases_res {
        Ok(purchases) => HttpResponse::Ok().json(purchases),
        Err(e) => HttpResponse::BadRequest().json(e)
    }
    

}