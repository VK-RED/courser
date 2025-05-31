use serde::{Deserialize, Serialize};
use sqlx::{Pool, Postgres};

use crate::{errors::CustomError, schema::{admin::CreateAdmin, StructWithId, StructWithVal}};

#[derive(Debug, Serialize, Deserialize)]
pub struct Admin{
    pub id: String,
    pub name: String,
    pub email: String,
    pub password: String,
}

pub async fn create_admin(pool:&Pool<Postgres>, admin_meta: CreateAdmin) -> Result<String, CustomError>{

    let user = sqlx::query_as!(
        StructWithId,
        r#"
            INSERT INTO admin_table (name, email, password)
            VALUES ($1, $2, $3)
            RETURNING id
        "#,
        admin_meta.name,
        admin_meta.email,
        admin_meta.password
    ).fetch_one(pool)
    .await;

    match user {
        Ok(val) => Ok(val.id),
        Err(_e) => Err(CustomError{error:"Error while creating admin".to_string()})
    }
}

pub async fn retrieve_admin_password(pool:&Pool<Postgres>, email:&str) -> Result<String, CustomError>{

    let res = sqlx::query_as!(
        StructWithVal,
        r#"
            SELECT password AS val FROM admin_table
            WHERE email = $1
        "#,
        email
    )
    .fetch_one(pool)
    .await
    .map_err(|_e|CustomError{error:"Error while retrieving admin password".to_string()})?;

    Ok(res.val)
}

pub async fn check_admin_exists(pool:&Pool<Postgres>, email:&str) -> Result<bool, CustomError>{

    let result = sqlx::query_as!(
        StructWithVal,
        r#"
            SELECT email as val FROM admin_table
            WHERE email = $1
        "#,
        email
    )
    .fetch_optional(pool)
    .await.map_err(|_e|CustomError{error:"Error while fetching admin by email".to_string()})?;

    match result {
        Some(_val) => Ok(true),
        None => Ok(false)
    }

}

pub async fn get_admin_id_by_email(pool:&Pool<Postgres>, email:&String) -> Result<String, CustomError>{

    let result = sqlx::query_as!(
        StructWithId,
        r#"
            SELECT id FROM admin_table
            WHERE email = $1
        "#,
        email
    ).fetch_one(pool)
    .await
    .map_err(|_|CustomError{error:"Error while fetching admin id".to_string()})?;

    Ok(result.id)
}