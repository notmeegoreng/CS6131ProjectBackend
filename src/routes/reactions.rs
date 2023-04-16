use tide::StatusCode;
use crate::Request;

pub async fn all_reactions(req: Request) -> tide::Result {
    sqlx::query!(
        "SELECT reaction FROM reactions"
    )
}

pub async fn add_reaction(mut req: Request) -> tide::Result {
    if let Some(user_id) = req.session().get::<u32>("user_id") {
        let post_id = req.param("post_id")?.parse::<u32>()?;
        let react = req.body_string().await?;
        tide::log::debug!("react: {} (len {})", react, react.len());
        if react.len() > 16 {
            return Ok(StatusCode::BadRequest.into())
        }
        sqlx::query!(
            "INSERT INTO reactions_user(post_id, reactor_id, reaction) VALUES (?, ?, ?)",
            post_id, user_id, react
        ).execute(&req.state().db).await?;

        Ok(StatusCode::Ok.into())
    } else {
        Ok(StatusCode::Unauthorized.into())
    }
}

pub async fn rem_reaction(mut req: Request) -> tide::Result {
    if let Some(user_id) = req.session().get::<u32>("user_id") {
        let post_id = req.param("post_id")?.parse::<u32>()?;
        let react = req.body_string().await?;
        tide::log::debug!("react: {} (len {})", react, react.len());
        if react.len() > 16 {
            return Ok(StatusCode::BadRequest.into())
        }
        sqlx::query!(
            "DELETE FROM reactions_user WHERE post_id = ? AND reactor_id = ? AND reaction = ?",
            post_id, user_id, react
        ).execute(&req.state().db).await?;
        Ok(StatusCode::Ok.into())
    } else {
        Ok(StatusCode::Unauthorized.into())
    }
}
