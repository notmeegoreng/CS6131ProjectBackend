use crate::Request;
use super::super::models;

use tide;
use serde_json;

pub async fn create(_req: Request) -> tide::Result {
    // let users
    Ok("".into())
}

pub async fn get(req: Request) -> tide::Result {
    let user = sqlx::query_as!(models::User, "SELECT user_id, username, description, profile_tag, is_admin as `is_admin: _` FROM users WHERE user_id = ?", req.param("user_id")?.parse::<i32>()?)
        .fetch_one(&req.state().db).await?;
    Ok(serde_json::to_string(&user)?.into())
}