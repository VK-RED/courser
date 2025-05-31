use serde::{Deserialize, Serialize};
use sqlx::{Pool, Postgres};

use crate::{errors::CustomError, schema::{user::CreateUser, StructWithId, StructWithVal}};

#[derive(Debug, Serialize, Deserialize)]
pub struct User{
    pub id: String,
    pub name: String,
    pub email: String,
    pub password: String,
}

pub async fn check_user_exists(pool:&Pool<Postgres>, email:&str) -> Result<bool, CustomError>{

    let result = sqlx::query_as!(
        StructWithVal,
        r#"
            SELECT email as val FROM user_table
            WHERE email = $1
        "#,
        email
    )
    .fetch_optional(pool)
    .await.map_err(|_e|CustomError{error:"Error while fetching user by email".to_string()})?;

    match result {
        Some(_val) => Ok(true),
        None => Ok(false)
    }

}   

pub async fn create_user(pool:&Pool<Postgres>, user_meta: CreateUser) -> Result<String, CustomError>{

    let user = sqlx::query_as!(
        StructWithId,
        r#"
            INSERT INTO user_table (name, email, password)
            VALUES ($1, $2, $3)
            RETURNING id
        "#,
        user_meta.name,
        user_meta.email,
        user_meta.password
    ).fetch_one(pool)
    .await;

    match user {
        Ok(val) => Ok(val.id),
        Err(_e) => Err(CustomError{error:"Error while creating user".to_string()})
    }
}

pub async fn retrieve_password(pool:&Pool<Postgres>, email:&str) -> Result<String, CustomError>{

    let res = sqlx::query_as!(
        StructWithVal,
        r#"
            SELECT password AS val FROM user_table
            WHERE email = $1
        "#,
        email
    )
    .fetch_one(pool)
    .await
    .map_err(|_e|CustomError{error:"Error while retrieving user password".to_string()})?;

    Ok(res.val)
}

pub async fn get_user_id_by_email(pool:&Pool<Postgres>, email:&String) -> Result<String, CustomError>{

    let result = sqlx::query_as!(
        StructWithId,
        r#"
            SELECT id FROM user_table
            WHERE email = $1
        "#,
        email
    ).fetch_one(pool)
    .await
    .map_err(|_|CustomError{error:"Error while fetching user id".to_string()})?;

    Ok(result.id)
}