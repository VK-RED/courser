use actix_web::{web, App, HttpServer};
use errors::AppError;
use handler::*;

mod handler;
mod errors;

#[actix_web::main]
async fn main() -> Result<(), AppError> {

    let address = "127.0.0.1:8080";

    println!("The Server is running at PORT : 8080");

    HttpServer::new(
        ||{
            App::new()
            .service(
                web::scope("/api/v1")
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
