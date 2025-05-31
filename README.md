# Courser

Courser is a platform that enables users to create, manage, and sell online courses. Built with performance and scalability in mind using Rust's robust ecosystem.

## Tech Stack

- Backend: Rust
- Web Framework: Actix
- Database: PostgreSQL
- SQL: Sqlx(Compile-Time & Runtime)

## Getting Started

- Clone the project ```https://github.com/VK-RED/courser.git```
- Copy env variables `cp .env.example .env`
- Install [sqlx-cli](!%5BImage%5D%28https://github.com/user-attachments/assets/3475e17d-158e-462a-a89a-3ceececb0c09%29) if not installed.
- Run `docker run -e POSTGRES_PASSWORD=postgres -p 5432:5432 postgres` to start the DB.
- Run `sqlx migrate run` to apply the migrations.
- Run ```cargo run``` to start the application.
- You can visit http://localhost:8080/api/v1/hello

## Tests
### Note: Tests run in parallel using threads and each test is independent of another
- To start the tests run `cargo test`
- To see the captured logs run `cargo test -- --show-output`
![Image](https://github.com/user-attachments/assets/3475e17d-158e-462a-a89a-3ceececb0c09)

## Folder Overview
- `migrations`  contains all the SQL migration history.
- `src/handlers` contains the route handlers.
- `src/middlewares` contains the user and admin middlewares.
- `src/models` contains the structure of DB model and its associated DB functions.
- `src/schema` contains incoming request and outgoing response schema.
- `src/test_init_app` contains the boiler plate init function to start the tests.
- `src/errors` contains the error responses.
- `src/utils` contains utility functions

### CONTRIBUTIONS
If you feel an issue or something needs to be fixed , please raise an Issue or a PR. Your contributions are welcomed most !! :pray: