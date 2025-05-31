use std::str::FromStr;

use actix_web::{get, post, web, HttpMessage, HttpRequest, HttpResponse, Responder};
use sqlx::types::{uuid, Uuid};

use crate::{errors::CustomError,models::{course, purchase::{self, get_user_purchases}, user::get_user_id_by_email}, schema::{admin::CourseResponse, PurchaseResponse, StructWithEmail}, GlobalState};

#[post("/{course_id}")]
async fn purchase_course_handler(data:web::Data<GlobalState>, path:web::Path<String>, req:HttpRequest) -> impl Responder {

    let pool = &data.pool;

    let user_email = req.extensions().get::<StructWithEmail>().cloned();

    if user_email.is_none(){
        return HttpResponse::Forbidden().json(CustomError{error:"email missing".to_string()});
    }

    let user_email = user_email.unwrap().email;

    let user_exists = get_user_id_by_email(pool, &user_email).await;

    if let Err(e) = user_exists{
        return HttpResponse::Forbidden().json(e);
    }

    let user_uuid = Uuid::from_str(&user_exists.unwrap());

    if user_uuid.is_err(){
        return HttpResponse::InternalServerError().json(CustomError{error:"Internal Error".to_string()});
    }

    let course_id = path.into_inner();
    let course_uuid = uuid::Uuid::from_str(&course_id);

    if course_uuid.is_err(){
        return HttpResponse::InternalServerError().json(CustomError{error:"Internal Error".to_string()});
    }

    let existing_purchases = get_user_purchases(pool, user_uuid.clone().unwrap()).await;

    if let Err(e) = existing_purchases{
        return HttpResponse::InternalServerError().json(e);
    }

    let existing_purchases = existing_purchases.unwrap();

    let existing_purchase = existing_purchases.into_iter().find(|purchase|{
        match purchase.course_id.to_string().as_str() == course_id.as_str(){
            true => true,
            false => false,
        }
    });

    if existing_purchase.is_some(){
        return HttpResponse::BadRequest().json(CustomError{error:"Already Purchased".to_string()});
    }

    let purchase_res = purchase::purchase_course(pool, course_uuid.unwrap(), user_uuid.unwrap()).await;

    match purchase_res {
        Ok(res) => HttpResponse::Ok().json(PurchaseResponse{id:res.id, message:"Purchased Successfully"}),
        Err(e) => HttpResponse::BadGateway().json(e),
    }

    
}

#[get("")]
pub async fn get_all_courses_handler(data:web::Data<GlobalState>) -> impl Responder {
    let pool = &data.pool;

    let courses = course::get_all_courses(pool).await;

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