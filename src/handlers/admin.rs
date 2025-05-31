use std::str::FromStr;

use actix_web::{get, post, put, web::{self, Json}, HttpMessage, HttpRequest, HttpResponse, Responder};
use chrono::{Duration, Utc};
use jsonwebtoken::{encode, EncodingKey, Header};
use sqlx::types::Uuid;

use crate::{errors::CustomError, models::{admin::{check_admin_exists, create_admin, get_admin_id_by_email, retrieve_admin_password}, course::{self, create_course}}, schema::{admin::{CourseResponse, CreateAdmin, CreateCourse, CreateCourseWithoutAdminId, UpdateCourse}, EmailAndPassword, JWTClaims, SigninResponse, SignupResponse}, utils::{hash_password, verify_password}, GlobalState};

#[post("/signup")]
async fn signup_admin(data:web::Data<GlobalState>, admin:Json<CreateAdmin>) -> impl Responder{
    let pool = &data.pool;
    let admin_exists = check_admin_exists(&data.pool, &admin.email).await;

    if let Err(e) = admin_exists{
        return HttpResponse::InternalServerError().json(e);
    }

    if admin_exists.unwrap() == true{
        return HttpResponse::BadRequest().json(CustomError{error:"User exists already with this email".to_string()});
    }

    let password_hash = hash_password(&admin.password);

    if let Err(_e) = password_hash{
        return HttpResponse::InternalServerError().json(CustomError{error:"Something went wrong !".to_string()});
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
        return HttpResponse::BadRequest().json(CustomError{error:"Signup first".to_string()});
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
        return HttpResponse::InternalServerError().json(CustomError{error:"Internal Error".to_string()});
    }

    match is_valid {
        Ok(()) => HttpResponse::Ok().json(SigninResponse{message:String::from("Signined in Successfully"), token:token.unwrap()}),
        Err(_) => HttpResponse::BadRequest().json(CustomError{error:"Enter Valid Password".to_string()}) 
    }

}

#[post("")]
async fn create_course_handler(data:web::Data<GlobalState>, course:Json<CreateCourseWithoutAdminId>, req:HttpRequest) -> impl Responder{
    let pool = &data.pool;

    let admin_email = req.extensions().get::<String>().cloned();

    if admin_email.is_none(){
        return HttpResponse::Forbidden().json(CustomError{error:"email missing".to_string()});
    }

    let admin_email = admin_email.unwrap();

    let admin_exists = get_admin_id_by_email(pool, &admin_email).await;

    if let Err(e) = admin_exists{
        return HttpResponse::InternalServerError().json(e);
    }

    let admin_uuid = Uuid::from_str(&admin_exists.unwrap());

    if admin_uuid.is_err(){
        return HttpResponse::InternalServerError().json(CustomError{error:"Internal Error".to_string()});
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

#[put("/{id}")]
async fn update_course_handler(data:web::Data<GlobalState>, course:Json<UpdateCourse>, req:HttpRequest, path:web::Path<String>) -> impl Responder {
    
    let pool = &data.pool;

    let course_id = path.into_inner();
    let course_uuid = Uuid::from_str(&course_id);

    if course_uuid.is_err(){
        return HttpResponse::InternalServerError().json(CustomError{error:"Internal Error".to_string()});
    }

    let admin_email = req.extensions().get::<String>().cloned();

    if admin_email.is_none(){
        return HttpResponse::Forbidden().json(CustomError{error:"email missing".to_string()});
    }

    let admin_email = admin_email.unwrap();

    let admin_exists = get_admin_id_by_email(pool, &admin_email).await;

    if let Err(e) = admin_exists{
        return HttpResponse::InternalServerError().json(e);
    }

    let admin_uuid = Uuid::from_str(&admin_exists.unwrap());

    if admin_uuid.is_err(){
        return HttpResponse::InternalServerError().json(CustomError{error:"Internal Error".to_string()});
    }

    let existing_course = course::get_course_by_id(pool, course_uuid.unwrap()).await;

    if let Err(e) = existing_course{
        return HttpResponse::InternalServerError().json(e);
    }

    let existing_course = existing_course.unwrap();

    if existing_course.admin_id != admin_uuid.unwrap(){
        return HttpResponse::Forbidden().json(CustomError{error:"Unauthorized".to_string()});
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

    let admin_email = req.extensions().get::<String>().cloned();

    if admin_email.is_none(){
        return HttpResponse::Forbidden().json(CustomError{error:"email missing".to_string()});
    }

    let admin_email = admin_email.unwrap();

    let admin_exists = get_admin_id_by_email(pool, &admin_email).await;

    if let Err(e) = admin_exists{
        return HttpResponse::InternalServerError().json(e);
    }

    let admin_uuid = Uuid::from_str(&admin_exists.unwrap());

    if admin_uuid.is_err(){
        return HttpResponse::InternalServerError().json(CustomError{error:"Internal Error".to_string()});
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

#[cfg(test)]
mod tests {
    use crate::{schema::admin::CreateCourseWithoutAdminId, test_init_app::init};
    use actix_web::test;
    use super::*;

    #[actix_web::test]
    async fn test_admin_signup_and_signin() {
        let (app, pool) = init(signup_admin).await;

        let admin = CreateAdmin {
            email: String::from("admin@test.com"),
            name: String::from("Test Admin"),
            password: String::from("adminpass123")
        };

        let res = test::TestRequest::post()
            .set_json(admin)
            .uri("/api/v1/admin/signup")
            .send_request(&app)
            .await;

        assert!(res.status().is_success());

        let signup_res_body: SignupResponse = test::read_body_json(res).await;
        assert_eq!(signup_res_body.message, "Signed up successfully".to_string());

        let json = EmailAndPassword {
            email: "admin@test.com".to_string(),
            password: "adminpass123".to_string(),
        };

        let res = test::TestRequest::post()
            .set_json(json)
            .uri("/api/v1/admin/signin")
            .send_request(&app)
            .await;

        assert!(res.status().is_success());

        let res_body: SigninResponse = test::read_body_json(res).await;
        assert_eq!(&res_body.message, "Signined in Successfully");

        sqlx::query("DELETE FROM admin_table WHERE email = $1")
            .bind("admin@test.com")
            .execute(&pool)
            .await
            .unwrap();
    }

    #[actix_web::test]
    async fn test_admin_invalid_credentials() {
        let (app, pool) = init(signup_admin).await;

        let admin = CreateAdmin {
            email: String::from("admin2@test.com"),
            name: String::from("Test Admin"),
            password: String::from("adminpass123")
        };

        let res = test::TestRequest::post()
            .set_json(admin)
            .uri("/api/v1/admin/signup")
            .send_request(&app)
            .await;

        assert!(res.status().is_success());

        let json = EmailAndPassword {
            email: "admin2@test.com".to_string(),
            password: "wrongpassword".to_string(),
        };

        let res = test::TestRequest::post()
            .set_json(json)
            .uri("/api/v1/admin/signin")
            .send_request(&app)
            .await;

        let res_body: CustomError = test::read_body_json(res).await;
        assert_eq!(res_body.error, "Enter Valid Password".to_string());

        sqlx::query("DELETE FROM admin_table WHERE email = $1")
            .bind("admin2@test.com")
            .execute(&pool)
            .await
            .unwrap();
    }

    #[actix_web::test]
    async fn test_create_course() {
        let (app, pool) = init(signup_admin).await;

        // First signup and signin to get token
        let admin = CreateAdmin {
            email: String::from("admin3@test.com"),
            name: String::from("Test Admin"),
            password: String::from("adminpass123")
        };

        let _ = test::TestRequest::post()
            .set_json(admin)
            .uri("/api/v1/admin/signup")
            .send_request(&app)
            .await;

        let json = EmailAndPassword {
            email: "admin3@test.com".to_string(),
            password: "adminpass123".to_string(),
        };

        let signin_res = test::TestRequest::post()
            .set_json(json)
            .uri("/api/v1/admin/signin")
            .send_request(&app)
            .await;

        let signin_body: SigninResponse = test::read_body_json(signin_res).await;
        let token = signin_body.token;

        // Create course
        let course = CreateCourseWithoutAdminId {
            title: "Test Course".to_string(),
            image_url: Some("https://test.com/image.jpg".to_string()),
            price: 5000,
        };

        let res = test::TestRequest::post()
            .set_json(course)
            .append_header(("Authorization", token))
            .uri("/api/v1/admin/course")
            .send_request(&app)
            .await;

        assert!(res.status().is_success());

        let course_res: CourseResponse = test::read_body_json(res).await;
        assert_eq!(course_res.title, "Test Course");

        // Cleanup
        sqlx::query("DELETE FROM course_table WHERE title = $1")
            .bind("Test Course")
            .execute(&pool)
            .await
            .unwrap();

        sqlx::query("DELETE FROM admin_table WHERE email = $1")
            .bind("admin3@test.com")
            .execute(&pool)
            .await
            .unwrap();
    }

    #[actix_web::test]
    async fn test_create_course_without_auth() {
        let (app, _) = init(create_course_handler).await;

        let course = CreateCourseWithoutAdminId {
            title: "Test Course".to_string(),
            image_url: Some("https://test.com/image.jpg".to_string()),
            price: 7888,
        };

        let req = test::TestRequest::post()
            .set_json(course)
            .uri("/api/v1/admin/course")
            .to_request();

        let res = test::try_call_service(&app, req).await;

        match res {
            Ok(_) => panic!("Expected error"),
            Err(e) => {
                let e= e.as_error::<CustomError>().unwrap();
                assert_eq!(e.error, "token missing");
            },
        }
    }

    #[actix_web::test]
    async fn test_update_course() {
        let (app, pool) = init(signup_admin).await;

        // First create an admin and get token
        let admin = CreateAdmin {
            email: String::from("admin4@test.com"),
            name: String::from("Test Admin"),
            password: String::from("adminpass123")
        };

        let _ = test::TestRequest::post()
            .set_json(admin)
            .uri("/api/v1/admin/signup")
            .send_request(&app)
            .await;

        let json = EmailAndPassword {
            email: "admin4@test.com".to_string(),
            password: "adminpass123".to_string(),
        };

        let signin_res = test::TestRequest::post()
            .set_json(json)
            .uri("/api/v1/admin/signin")
            .send_request(&app)
            .await;

        let signin_body: SigninResponse = test::read_body_json(signin_res).await;
        let token = signin_body.token;

        // First create a course
        let course = CreateCourseWithoutAdminId {
            title: "Original Course".to_string(),
            image_url: Some("https://test.com/old.jpg".to_string()),
            price: 5000,
        };

        let create_res = test::TestRequest::post()
            .set_json(course)
            .append_header(("Authorization", token.clone()))
            .uri("/api/v1/admin/course")
            .send_request(&app)
            .await;

        let created_course: CourseResponse = test::read_body_json(create_res).await;

        // Now update the course
        let update_course = UpdateCourse {
            title: "Updated Course".to_string(),
            image_url: Some("https://test.com/new.jpg".to_string()),
            price: 6000,
        };

        let update_res = test::TestRequest::put()
            .set_json(update_course)
            .append_header(("Authorization", token))
            .uri(&format!("/api/v1/admin/course/{}", created_course.id))
            .send_request(&app)
            .await;

        assert!(update_res.status().is_success());

        let updated_course: CourseResponse = test::read_body_json(update_res).await;
        assert_eq!(updated_course.title, "Updated Course");
        assert_eq!(updated_course.price, 6000);

        let course_uuid = Uuid::from_str(&created_course.id).unwrap();

        // Cleanup
        sqlx::query("DELETE FROM course_table WHERE id = $1")
            .bind(course_uuid)
            .execute(&pool)
            .await
            .unwrap();

        sqlx::query("DELETE FROM admin_table WHERE email = $1")
            .bind("admin4@test.com")
            .execute(&pool)
            .await
            .unwrap();
    }

    #[actix_web::test]
    async fn test_get_all_courses() {
        let (app, pool) = init(signup_admin).await;

        // Create admin and get token
        let admin = CreateAdmin {
            email: String::from("admin7@test.com"),
            name: String::from("Test Admin"),
            password: String::from("adminpass123")
        };

        let _ = test::TestRequest::post()
            .set_json(admin)
            .uri("/api/v1/admin/signup")
            .send_request(&app)
            .await;

        let json = EmailAndPassword {
            email: "admin7@test.com".to_string(),
            password: "adminpass123".to_string(),
        };

        let signin_res = test::TestRequest::post()
            .set_json(json)
            .uri("/api/v1/admin/signin")
            .send_request(&app)
            .await;

        let signin_res_body: SigninResponse = test::read_body_json(signin_res).await;

        let token = signin_res_body.token;

        // Create multiple courses
        let courses = vec![
            CreateCourseWithoutAdminId {
                title: "Course 1".to_string(),
                image_url: Some("https://test.com/1.jpg".to_string()),
                price: 5000,
            },
            CreateCourseWithoutAdminId {
                title: "Course 2".to_string(),
                image_url: Some("https://test.com/2.jpg".to_string()),
                price: 6000,
            },
        ];

        let mut created_courses = Vec::new();

        for course in courses {
            let create_res = test::TestRequest::post()
                .set_json(course)
                .append_header(("Authorization", token.clone()))
                .uri("/api/v1/admin/course")
                .send_request(&app)
                .await;

            let course: CourseResponse = test::read_body_json(create_res).await;
            created_courses.push(course);
        }

        // Get all courses
        let get_res = test::TestRequest::get()
            .append_header(("Authorization", token))
            .uri("/api/v1/admin/course/courses")
            .send_request(&app)
            .await;

        assert!(get_res.status().is_success());
        let courses: Vec<CourseResponse> = test::read_body_json(get_res).await;
        assert_eq!(courses.len(), 2);
        assert!(courses.iter().any(|c| c.title == "Course 1"));
        assert!(courses.iter().any(|c| c.title == "Course 2"));

        // Cleanup
        for course in created_courses {

            let course_uuid = Uuid::from_str(&course.id).unwrap();

            sqlx::query("DELETE FROM course_table WHERE id = $1")
                .bind(course_uuid)
                .execute(&pool)
                .await
                .unwrap();
        }

        sqlx::query("DELETE FROM admin_table WHERE email = $1")
            .bind("admin7@test.com")
            .execute(&pool)
            .await
            .unwrap();
    }

}