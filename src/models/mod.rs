use serde::{Serialize, Deserialize};
use sqlx;

#[derive(Debug, Eq, PartialEq, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    pub user_id: i32,
    pub username: String,
    pub description: String,
    pub profile_tag: String,
    // password hash is not stored on the main struct as it should only be used on login
    pub is_admin: bool
}

#[derive(Debug, Eq, PartialEq, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Log {
    pub log_id: i32,
    pub user_id: i32,
    pub log: String,
}

#[derive(Debug, Eq, PartialEq, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Category {
    pub category_id: i32,
    pub name: String,
    pub description: String
}