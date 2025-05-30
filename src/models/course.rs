use sqlx::{types::{uuid, Uuid}, Pool, Postgres};

use crate::{errors::CustomError, schema::{admin::{CreateCourse, UpdateCourse}, StructWithId}};

#[derive(Debug)]
pub struct Course{
    pub id: String,
    pub title: String,
    pub image_url: Option<String>,
    pub price: i32,
    pub admin_id: uuid::Uuid,
}

pub async fn create_course(pool:&Pool<Postgres>, course_details:CreateCourse) -> Result<Course, CustomError>{
    let result = sqlx::query_as!(
        Course,
        r#"
            INSERT INTO course_table (title, image_url, price, admin_id)
            VALUES ($1, $2, $3, $4)  
            RETURNING *
        "#,
        course_details.title,
        course_details.image_url,
        course_details.price,
        course_details.admin_id,
    )
    .fetch_one(pool)
    .await;

    match result {
        Ok(val) => Ok(val),
        Err(_) => Err(CustomError { error: "Error while creating a course" })
    }
}

pub async fn get_course_by_id(pool:&Pool<Postgres>, id:Uuid)->Result<Course, CustomError>{

    let result = sqlx::query_as!(
        Course,
        r#"
            SELECT * FROM course_table
            WHERE id = $1
        "#,
        id
    )
    .fetch_one(pool)
    .await;

    match result {
        Ok(val) => Ok(val),
        Err(_) => Err(CustomError { error: "Error while fetching the course" })
    }

}

pub async fn update_course(pool:&Pool<Postgres>, updated_course: UpdateCourse) -> Result<Course, CustomError>{
    let result = sqlx::query_as!(
        Course,
        r#"
            UPDATE course_table
            SET title = $1, image_url = $2, price = $3
            RETURNING *
        "#,
        updated_course.title,
        updated_course.image_url,
        updated_course.price
    )
    .fetch_one(pool)
    .await;

    match result {
        Ok(val) => Ok(val),
        Err(_) => Err(CustomError { error: "Error while updating the course" })
    }
}

pub async fn get_all_admin_courses(pool:&Pool<Postgres>, admin_id:Uuid) -> Result<Vec<Course>, CustomError>{

    let result = sqlx::query_as!(
        Course,
        r#"
            SELECT * FROM course_table
            WHERE admin_id = $1
        "#,
        admin_id
    )
    .fetch_all(pool)
    .await;

    match result {
        Ok(val) => Ok(val),
        Err(_) => Err(CustomError { error: "Error while fetching all the courses" })
    }
}

pub async fn get_all_courses(pool:&Pool<Postgres>) -> Result<Vec<Course>, CustomError>{

    let result = sqlx::query_as!(
        Course,
        r#"
            SELECT * FROM course_table
        "#,
    )
    .fetch_all(pool)
    .await;

    match result {
        Ok(val) => Ok(val),
        Err(_) => Err(CustomError { error: "Error while fetching all the courses" })
    }
}
