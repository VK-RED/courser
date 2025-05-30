use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct CreateUser{
    pub name: String,
    pub email: String,
    pub password: String,
}