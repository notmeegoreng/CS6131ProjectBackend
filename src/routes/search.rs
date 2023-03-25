use serde::Deserialize;

use crate::Request;
use crate::models::Container;
use crate::routes::users::User;

#[derive(Deserialize)]
struct SearchQuery {
    q: Option<String>
}

pub async fn category_search(req: Request) -> tide::Result {
    let query = req.query::<SearchQuery>()?;
    let result;
    if let Some(q) = query.q {
        result = sqlx::query_as!(
            Container,
            "SELECT category_id AS id, name, description FROM categories WHERE '%' + ? + '%'",
            q
        ).fetch_all(&req.state().db).await?;
    } else {
        result = sqlx::query_as!(
            Container, "SELECT category_id AS id, name, description FROM categories"
        ).fetch_all(&req.state().db).await?;
    }
    Ok(serde_json::to_value(result)?.into())
}

pub async fn forum_search(req: Request) -> tide::Result {
    let query = req.query::<SearchQuery>()?;
    let result;
    if let Some(q) = query.q {
        result = sqlx::query_as!(
            Container,
            "SELECT forum_id AS id, name, description FROM forums WHERE name LIKE ('%' + ? + '%')",
            q
        ).fetch_all(&req.state().db).await?;
    } else {
        result = sqlx::query_as!(
            Container, "SELECT forum_id AS id, name, description FROM forums"
        ).fetch_all(&req.state().db).await?;
    }
    Ok(serde_json::to_value(result)?.into())
}

pub async fn topic_search(req: Request) -> tide::Result {
    let query = req.query::<SearchQuery>()?;
    let result;
    if let Some(q) = query.q {
        result = sqlx::query_as!(
            Container,
            "SELECT topic_id AS id, name, description FROM topics WHERE name LIKE ('%' + ? + '%')",
            q
        ).fetch_all(&req.state().db).await?;
    } else {
        result = sqlx::query_as!(
            Container, "SELECT topic_id AS id, name, description FROM topics"
        ).fetch_all(&req.state().db).await?;
    }
    Ok(serde_json::to_value(result)?.into())
}

pub async fn user_search(req: Request) -> tide::Result {
    let query = req.query::<SearchQuery>()?;
    let result;
    if let Some(q) = query.q {
        result = sqlx::query_as!(
            User,
            "SELECT user_id, username, profile_tag, description,
             is_avatar_set AS `is_avatar_set: _`, is_admin AS `is_admin: _`
             FROM users WHERE username LIKE ('%' + ? + '%')",
            q
        ).fetch_all(&req.state().db).await?;
    } else {
        result = sqlx::query_as!(
            User,
            "SELECT user_id, username, profile_tag, description,
             is_avatar_set AS `is_avatar_set: _`, is_admin AS `is_admin: _`
             FROM users"
        ).fetch_all(&req.state().db).await?;
    }
    Ok(serde_json::to_value(result)?.into())
}

