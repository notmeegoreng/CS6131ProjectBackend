use tide::{Response, StatusCode};
use serde::{Serialize, Deserialize};
use serde_json;

use crate::{Request, utils::route_get};

#[derive(Debug, Eq, PartialEq, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    pub user_id: u32,
    pub username: String,
    pub description: String,
    pub profile_tag: String,
    // password hash is not stored on the main struct as it should only be used on login
    pub is_avatar_set: bool,
    pub is_admin: bool
}

#[derive(Debug, Eq, PartialEq, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Log {
    pub log_id: u32,
    pub log: String,
    pub time: chrono::NaiveDateTime
}

#[derive(Deserialize)]
struct UsernameQuery {
    username: String
}

pub async fn available_username(req: Request) -> tide::Result {
    match sqlx::query!(
        "SELECT 1 AS available FROM users WHERE username = ?", req.query::<UsernameQuery>()?.username
    ).fetch_optional(&req.state().db).await? {
        Some(_) => Ok(Response::new(StatusCode::Conflict)),
        None => Ok("".into())
    }
}

/*
pub async fn get(req: Request) -> tide::Result {
    let user = sqlx::query_as!(
        models::User,
        "SELECT user_id, username, description, profile_tag, is_admin AS `is_admin: _` FROM users WHERE user_id = ?",
        req.param("user_id")?.parse::<i32>()?)
        .fetch_one(&req.state().db).await?;
    Ok(serde_json::to_string(&user)?.into())
}*/
route_get!(
    user_get, req, User,
    "SELECT user_id, username, description, profile_tag, is_avatar_set AS `is_avatar_set: _`, is_admin AS `is_admin: _` FROM users WHERE user_id = ?",
    req.param("user_id")?.parse::<u32>()?
);

pub async fn log_get(req: Request) -> tide::Result {
    let user_id = req.param("user_id")?.parse::<u32>()?;
    let data = sqlx::query_as!(Log,
        "SELECT log_id, log, time FROM audit_log WHERE user_id = ?",
        user_id).fetch_all(&req.state().db).await?;
    if data.len() != 0 || sqlx::query!(
        "SELECT is_admin AS `is_admin: bool` FROM users WHERE user_id = ?", user_id
    ).fetch_one(&req.state().db).await?.is_admin {
        return Ok(serde_json::to_value(data)?.into())
    }
    Ok(Response::new(StatusCode::BadRequest))
}
