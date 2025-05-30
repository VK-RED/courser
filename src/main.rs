use actix_web::{web, App, HttpServer};
use errors::AppError;
use handler::*;
use dotenv::dotenv;
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};

mod handler;
mod errors;

struct GlobalState{
    pool: Pool<Postgres>
}

#[actix_web::main]
async fn main() -> Result<(), AppError> {

    dotenv().ok();

    let address = "127.0.0.1:8080";
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE URL must be set");

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
                web::scope("/api/v1")
                .app_data(app_data.clone())
                .service(hello_world)
            )
        }
    ).bind(address)
    .map_err(|_e|AppError::SocketBind)?
    .run()
    .await
    .map_err(|_e|AppError::ServerStart)?;

    Ok(())
    
}
