use std::str::FromStr;

use actix_web::{get, post, put, web::{self, Json}, HttpMessage, HttpRequest, HttpResponse, Responder};
use chrono::{Duration, Utc};
use jsonwebtoken::{encode, EncodingKey, Header};
use sqlx::types::Uuid;

use crate::{errors::CustomError, models::{admin::{check_admin_exists, create_admin, get_admin_id_by_email, retrieve_admin_password}, course::{self, create_course}}, schema::{admin::{CourseResponse, CreateAdmin, CreateCourse, CreateCourseWithoutAdminId, UpdateCourse}, EmailAndPassword, JWTClaims, SigninResponse, SignupResponse, StructWithEmail}, utils::{hash_password, verify_password}, GlobalState};

#[post("/signup")]
async fn signup_admin(data:web::Data<GlobalState>, admin:Json<CreateAdmin>) -> impl Responder{
    let pool = &data.pool;
    let admin_exists = check_admin_exists(&data.pool, &admin.email).await;

    if let Err(e) = admin_exists{
        return HttpResponse::InternalServerError().json(e);
    }

    if admin_exists.unwrap() == true{
        return HttpResponse::BadRequest().json(CustomError{error:"User exists already with this email"});
    }

    let password_hash = hash_password(&admin.password);

    if let Err(_e) = password_hash{
        return HttpResponse::InternalServerError().json(CustomError{error:"Something went wrong !"});
    }

    let admin_meta = CreateAdmin{
        email: admin.email.clone(),
        name: admin.name.clone(),
        password: password_hash.unwrap()
    };

    let signup_result = create_admin(pool, admin_meta).await;

    match signup_result {
        Ok(res) => HttpResponse::Ok().json(SignupResponse{message:String::from("Signed up successfully"),id: res}),
        Err(e) => HttpResponse::BadGateway().json(e),
    }
}

#[post("/signin")]
async fn signin_admin(data:web::Data<GlobalState>, admin_data:web::Json<EmailAndPassword>) -> impl Responder {

    let user_exists = check_admin_exists(&data.pool, &admin_data.email).await;

    if let Err(e) = user_exists{
        return HttpResponse::InternalServerError().json(e);
    }

    // throw when user not found
    if user_exists.unwrap() == false {
        return HttpResponse::BadRequest().json(CustomError{error:"Signup first"});
    }

    let pool = &data.pool;
    let password_hash_res = retrieve_admin_password(pool, &admin_data.email).await;

    if let Err(e) = password_hash_res{
        return HttpResponse::InternalServerError().json(e);
    }

    let hash = password_hash_res.unwrap();
    let is_valid = verify_password(&admin_data.password, &hash);

    let jwt_secret = std::env::var("ADMIN_JWT_PASSWORD").unwrap();

    let tomorrow = Utc::now() + Duration::days(1);

    let claims = JWTClaims{
        sub: admin_data.email.clone(),
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

#[post("/course")]
async fn create_course_handler(data:web::Data<GlobalState>, course:Json<CreateCourseWithoutAdminId>, req:HttpRequest) -> impl Responder{
    let pool = &data.pool;

    let admin_email = req.extensions().get::<StructWithEmail>().cloned();

    if admin_email.is_none(){
        return HttpResponse::Forbidden().json(CustomError{error:"email missing"});
    }

    let admin_email = admin_email.unwrap().email;

    let admin_exists = get_admin_id_by_email(pool, &admin_email).await;

    if let Err(e) = admin_exists{
        return HttpResponse::InternalServerError().json(e);
    }

    let admin_uuid = Uuid::from_str(&admin_exists.unwrap());

    if admin_uuid.is_err(){
        return HttpResponse::InternalServerError().json(CustomError{error:"Internal Error"});
    }

    let course = CreateCourse{
        title: course.title.clone(),
        image_url: course.image_url.clone(),
        price: course.price,
        admin_id: admin_uuid.unwrap(),
    };

    let course_res = create_course(pool, course).await;

    match course_res {
        Ok(res) => {
            let parsed_course = CourseResponse{
                id: res.id,
                admin_id: res.admin_id.to_string(),
                title: res.title,
                image_url: res.image_url,
                price: res.price,
            };

            HttpResponse::Ok().json(parsed_course)
        },
        Err(e) => HttpResponse::BadGateway().json(e),
    }

}

#[put("/course/{id}")]
async fn update_course_handler(data:web::Data<GlobalState>, course:Json<UpdateCourse>, req:HttpRequest, path:web::Path<String>) -> impl Responder {
    
    let pool = &data.pool;

    let course_id = path.into_inner();
    let course_uuid = Uuid::from_str(&course_id);

    if course_uuid.is_err(){
        return HttpResponse::InternalServerError().json(CustomError{error:"Internal Error"});
    }

    let admin_email = req.extensions().get::<StructWithEmail>().cloned();

    if admin_email.is_none(){
        return HttpResponse::Forbidden().json(CustomError{error:"email missing"});
    }

    let admin_email = admin_email.unwrap().email;

    let admin_exists = get_admin_id_by_email(pool, &admin_email).await;

    if let Err(e) = admin_exists{
        return HttpResponse::InternalServerError().json(e);
    }

    let admin_uuid = Uuid::from_str(&admin_exists.unwrap());

    if admin_uuid.is_err(){
        return HttpResponse::InternalServerError().json(CustomError{error:"Internal Error"});
    }

    let existing_course = course::get_course_by_id(pool, course_uuid.unwrap()).await;

    if let Err(e) = existing_course{
        return HttpResponse::InternalServerError().json(e);
    }

    let existing_course = existing_course.unwrap();

    if existing_course.admin_id != admin_uuid.unwrap(){
        return HttpResponse::Forbidden().json(CustomError{error:"Unauthorized"});
    }

    let course = UpdateCourse{
        title: course.title.clone(),
        image_url: course.image_url.clone(),
        price: course.price,
    };

    let course_res = course::update_course(pool, course).await;

    match course_res {
        Ok(res) => {
            let parsed_course = CourseResponse{
                id: res.id,
                admin_id: res.admin_id.to_string(),
                title: res.title,
                image_url: res.image_url,
                price: res.price,
            };

            HttpResponse::Ok().json(parsed_course)
        },
        Err(e) => HttpResponse::BadGateway().json(e),
    }

}

#[get("/courses")]
async fn get_all_courses_handler(data:web::Data<GlobalState>, req:HttpRequest) -> impl Responder {
    let pool = &data.pool;

    let admin_email = req.extensions().get::<StructWithEmail>().cloned();

    if admin_email.is_none(){
        return HttpResponse::Forbidden().json(CustomError{error:"email missing"});
    }

    let admin_email = admin_email.unwrap().email;

    let admin_exists = get_admin_id_by_email(pool, &admin_email).await;

    if let Err(e) = admin_exists{
        return HttpResponse::InternalServerError().json(e);
    }

    let admin_uuid = Uuid::from_str(&admin_exists.unwrap());

    if admin_uuid.is_err(){
        return HttpResponse::InternalServerError().json(CustomError{error:"Internal Error"});
    }

    let courses = course::get_all_admin_courses(pool, admin_uuid.unwrap()).await;

    match courses {
        Ok(courses) => {
            let parsed_courses = courses.into_iter().map(|course|{
                CourseResponse{
                    id: course.id,
                    admin_id: course.admin_id.to_string(),
                    title: course.title,
                    image_url: course.image_url,
                    price: course.price,
                }
            }).collect::<Vec<CourseResponse>>();

            HttpResponse::Ok().json(parsed_courses)
        },
        Err(e) => HttpResponse::BadGateway().json(e),
    }
}