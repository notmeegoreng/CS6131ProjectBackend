use tide::{Response, StatusCode};
use serde::{Deserialize};
use serde_json;

use crate::{Request, utils::route_get, models::{User, Log}};

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
    "SELECT user_id, username, description, profile_tag,
     is_avatar_set AS `is_avatar_set: _`, is_admin AS `is_admin: _`
     FROM users WHERE user_id = ?",
    req.param("user_id")?.parse::<u32>()?
);

#[derive(Deserialize)]
struct UserPatch {
    profile_tag: Option<String>,
    description: Option<String>
}

pub async fn user_patch(mut req: Request) -> tide::Result {
    let data: UserPatch = req.body_json().await?;
    if let Some(user_id) = req.session().get::<u32>("user_id") {
        sqlx::query!(
            "UPDATE users SET profile_tag = COALESCE(?, profile_tag),
             description = COALESCE(?, description) WHERE user_id = ?",
            data.profile_tag, data.description, user_id
        ).execute(&req.state().db).await?;
        return Ok(Response::new(StatusCode::NoContent));
    }
    Ok(Response::new(StatusCode::Unauthorized))
}

pub async fn log_get(req: Request) -> tide::Result {
    let user_id = req.param("user_id")?.parse::<u32>()?;
    let data = sqlx::query_as!(Log,
        "SELECT log_id, log, time FROM audit_log WHERE user_id = ? ORDER BY log_id DESC",
        user_id).fetch_all(&req.state().db).await?;
    if data.len() != 0 || sqlx::query!(
        "SELECT is_admin AS `is_admin: bool` FROM users WHERE user_id = ?", user_id
    ).fetch_one(&req.state().db).await?.is_admin {
        return Ok(serde_json::to_value(data)?.into())
    }
    Ok(Response::new(StatusCode::BadRequest))
}
