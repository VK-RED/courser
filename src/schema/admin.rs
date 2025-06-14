use serde::{Deserialize, Serialize};
use sqlx::types::Uuid;

#[derive(Debug, Deserialize, Serialize)]
pub struct CreateAdmin{
    pub name: String,
    pub email: String,
    pub password: String,
}
pub struct CreateCourse {
    pub title: String,
    pub image_url: Option<String>,
    pub price: i32,
    pub admin_id: Uuid,
}

#[derive(Deserialize, Serialize)]
pub struct CreateCourseWithoutAdminId {
    pub title: String,
    pub image_url: Option<String>,
    pub price: i32,
}

#[derive(Deserialize, Serialize)]
pub struct UpdateCourse {
    pub title: String,
    pub image_url: Option<String>,
    pub price: i32,
}

#[derive(Serialize, Deserialize)]
pub struct CourseResponse{
    pub id: String,
    pub title: String,
    pub image_url: Option<String>,
    pub price: i32,
    pub admin_id: String,
}