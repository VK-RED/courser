use actix_web::{test::{self}, App, web, dev::{HttpServiceFactory, ServiceResponse}, Error};
use actix_service::Service;
use actix_http::{Request};
use crate::{handlers, middlewares, GlobalState};
use dotenv::dotenv;
use actix_web::{middleware::from_fn, web::scope};
use sqlx::postgres::PgPoolOptions;

#[cfg(test)]
pub async fn init(_service_factory: impl HttpServiceFactory + 'static) -> impl Service<Request, Response = ServiceResponse, Error = Error> {

    dotenv().ok();

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE URL must be set");

    std::env::var("USER_JWT_PASSWORD").expect("USER_JWT_PASSWORD must be set");
    std::env::var("ADMIN_JWT_PASSWORD").expect("ADMIN_JWT_PASSWORD must be set");

    let pool = PgPoolOptions::new()
    .max_connections(5)
    .connect(&database_url)
    .await
    .expect("Cant connect to the database");

    let global_state = GlobalState{pool};

    let app_data = web::Data::new(global_state);

    test::init_service(
        App::new()
            .service(
                scope("/api/v1")
                .app_data(app_data.clone())
                .service(handlers::hello_world)
                // place this before /user , else other will get matched
                .service(
                    scope("/user/purchases")
                    .wrap(from_fn(middlewares::user::user_middleware))
                    .service(handlers::user::user_purchases)
                )
                .service(
                    scope("/user")
                    .service(handlers::user::signup_user)
                    .service(handlers::user::signin_user)
                )
                .service(
                    // guard the purchase handler
                    scope("/courses/purchase")
                    .wrap(from_fn(middlewares::user::user_middleware))
                    .service(handlers::course::purchase_course_handler)
                )
                .service(
                    //this will match /course/{id} as well as /courses
                    scope("/admin/course")
                    .wrap(from_fn(middlewares::admin::admin_middleware))
                    .service(handlers::admin::create_course_handler)
                    .service(handlers::admin::update_course_handler)
                    .service(handlers::admin::get_all_courses_handler)
                )
                .service(
                    scope("/admin")
                    .service(handlers::admin::signup_admin)
                    .service(handlers::admin::signin_admin)
                )
                .service(
                    scope("/courses")
                    .service(handlers::course::get_all_courses_handler)
                )
            )
    ).await
}
