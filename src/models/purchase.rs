use serde::{Deserialize, Serialize};
use sqlx::{types::Uuid, Pool, Postgres};

use crate::{errors::CustomError, schema::StructWithId};

#[derive(Serialize, Deserialize, Debug)]
pub struct Purchase{
    pub id: String,
    pub user_id: String,
    pub course_id: String,
}

pub async fn get_user_purchases(pool:&Pool<Postgres>, user_id:Uuid) -> Result<Vec<Purchase>, CustomError>{

    let user_purchases = sqlx::query_as!(
        Purchase,
        r#"
            SELECT id, user_id, course_id FROM purchases_table 
            WHERE user_id = $1
        "#,
        user_id
    )
    .fetch_all(pool)
    .await
    .map_err(|_e|CustomError{error:"Error while fetching user purchases"})?;

    Ok(user_purchases)
}

pub async fn purchase_course(pool:&Pool<Postgres>, course_id:Uuid, user_id:Uuid) -> Result<StructWithId, CustomError>{

    let result = sqlx::query_as!(
        StructWithId,
        r#"
            INSERT INTO purchases_table (user_id, course_id)
            VALUES ($1, $2)
            RETURNING id
        "#,
        user_id,
        course_id
    )
    .fetch_one(pool)
    .await;

    match result {
        Ok(val) => Ok(val),
        Err(_) => Err(CustomError { error: "Error while purchasing the course" })
    }
}