use actix_web::{middleware::from_fn, web::{self, scope}, App, HttpServer};
use errors::AppError;
use dotenv::dotenv;
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};

mod errors;
mod models;
mod schema;
mod handlers;
mod utils;
mod middlewares;

struct GlobalState{
    pool: Pool<Postgres>
}

#[actix_web::main]
async fn main() -> Result<(), AppError> {

    dotenv().ok();

    let address = "127.0.0.1:8080";
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE URL must be set");

    std::env::var("USER_JWT_PASSWORD").expect("USER_JWT_PASSWORD must be set");
    std::env::var("ADMIN_JWT_PASSWORD").expect("ADMIN_JWT_PASSWORD must be set");

    let pool = PgPoolOptions::new()
    .max_connections(5)
    .connect(&database_url)
    .await
    .map_err(|_e| AppError::DbConnect)?;

    let global_state = GlobalState{pool};

    let app_data = web::Data::new(global_state);

    println!("The Server is running at PORT : 8080");

    HttpServer::new(
        move||{
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
                    scope("/admin")
                    .service(handlers::admin::signup_admin)
                    .service(handlers::admin::signin_admin)
                )
                // TODO: add admin middleware and other routes
                .service(
                    // guard the purchase handler
                    scope("/courses/purchase")
                    .wrap(from_fn(middlewares::user::user_middleware))
                    .service(handlers::course::purchase_course_handler)
                )
                .service(
                    scope("/courses")
                    .service(handlers::course::get_all_courses_handler)
                )
            )
        }
    ).bind(address)
    .map_err(|_e|AppError::SocketBind)?
    .run()
    .await
    .map_err(|_e|AppError::ServerStart)?;

    Ok(())
    
}
