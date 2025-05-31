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
        Ok(res) => HttpResponse::Ok().json(PurchaseResponse{id:res.id, message:"Purchased Successfully".to_string()}),
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

#[cfg(test)]
mod tests {
    use crate::{models::purchase::Purchase, schema::{admin::{CreateAdmin, CreateCourseWithoutAdminId}, user::CreateUser, EmailAndPassword, SigninResponse}, test_init_app::init};
    use actix_web::test;
    use super::*;

    #[actix_web::test]
    async fn test_purchase_course() {
        let (app, pool) = init(get_all_courses_handler).await;

        // 1. Create an admin
        let admin = CreateAdmin {
            email: String::from("admin_purchase@test.com"),
            name: String::from("Test Admin"),
            password: String::from("adminpass123")
        };

        let _ = test::TestRequest::post()
            .set_json(admin)
            .uri("/api/v1/admin/signup")
            .send_request(&app)
            .await;

        // Sign in admin to get token
        let admin_creds = EmailAndPassword {
            email: "admin_purchase@test.com".to_string(),
            password: "adminpass123".to_string(),
        };

        let signin_res = test::TestRequest::post()
            .set_json(admin_creds)
            .uri("/api/v1/admin/signin")
            .send_request(&app)
            .await;

        let admin_signin_res : SigninResponse = test::read_body_json(signin_res).await;
        let admin_token = admin_signin_res.token;

        // 2. Create a course
        let course = CreateCourseWithoutAdminId {
            title: "Test Purchase Course".to_string(),
            image_url: Some("https://test.com/image.jpg".to_string()),
            price: 5000,
        };

        let create_course_res = test::TestRequest::post()
            .set_json(course)
            .append_header(("Authorization", admin_token))
            .uri("/api/v1/admin/course")
            .send_request(&app)
            .await;

        let course_res: CourseResponse = test::read_body_json(create_course_res).await;

        // 3. Create a user
        let user = CreateUser {
            email: String::from("user_purchase@test.com"),
            name: String::from("Test User"),
            password: String::from("userpass123")
        };

        let _ = test::TestRequest::post()
            .set_json(user)
            .uri("/api/v1/user/signup")
            .send_request(&app)
            .await;

        // Sign in user to get token
        let user_creds = EmailAndPassword {
            email: "user_purchase@test.com".to_string(),
            password: "userpass123".to_string(),
        };

        let user_signin_res = test::TestRequest::post()
            .set_json(user_creds)
            .uri("/api/v1/user/signin")
            .send_request(&app)
            .await;

        let user_signin_res : SigninResponse = test::read_body_json(user_signin_res).await;

        let user_token = user_signin_res.token;

        let uri = format!("/api/v1/courses/purchase/{}", course_res.id);
        println!("uri : {}", uri);

        // 4. Purchase the course
        let purchase_res = test::TestRequest::post()
            .append_header(("Authorization", user_token.clone()))
            .uri(&uri)
            .send_request(&app)
            .await;

        assert!(purchase_res.status().is_success());

        let purchase_body: PurchaseResponse = test::read_body_json(purchase_res).await;
        assert_eq!(purchase_body.message, "Purchased Successfully");

        // 5. Verify purchase exists in user's purchases
        let user_purchases_res = test::TestRequest::get()
            .append_header(("Authorization", user_token.clone()))
            .uri("/api/v1/user/purchases")
            .send_request(&app)
            .await;

        let purchases: Vec<Purchase> = test::read_body_json(user_purchases_res).await;
        assert_eq!(purchases.len(), 1);
        assert_eq!(purchases[0].course_id.to_string(), course_res.id);

        // 6. Try purchasing same course again (should fail)
        let repeat_purchase_res = test::TestRequest::post()
            .append_header(("Authorization", user_token.clone()))
            .uri(&uri)
            .send_request(&app)
            .await;

        let error_body: CustomError = test::read_body_json(repeat_purchase_res).await;
        assert_eq!(error_body.error, "Already Purchased");

        // Cleanup
        let course_uuid = Uuid::from_str(&course_res.id).unwrap();
        
        sqlx::query("DELETE FROM purchases_table WHERE course_id = $1")
            .bind(course_uuid)
            .execute(&pool)
            .await
            .unwrap();

        sqlx::query("DELETE FROM course_table WHERE id = $1")
            .bind(course_uuid)
            .execute(&pool)
            .await
            .unwrap();

        sqlx::query("DELETE FROM user_table WHERE email = $1")
            .bind("user_purchase@test.com")
            .execute(&pool)
            .await
            .unwrap();

        sqlx::query("DELETE FROM admin_table WHERE email = $1")
            .bind("admin_purchase@test.com")
            .execute(&pool)
            .await
            .unwrap();
    }
}