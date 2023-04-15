use tide::StatusCode;
use crate::Request;

pub async fn add_reaction(mut req: Request) -> tide::Result {
    if let Some(user_id) = req.session().get::<u32>("user_id") {
        let post_id = req.param("post_id")?.parse::<u32>()?;
        let react = req.body_string().await?;
        if (react.len()) != 1 {
            return Ok(StatusCode::BadRequest.into())
        }
        let r = sqlx::query!(
            "INSERT INTO added_reactions(post_id, reactor_id, reaction) VALUES (?, ?, ?)",
            post_id, user_id, react
        ).execute(&req.state().db).await;
        if let Err(sqlx::Error::Database(e)) = r {
            tide::log::debug!("DB Err: {}", e)
        } else if let Err(e) = r {
            tide::log::debug!("Other Err: {}", e)
        }

        Ok(StatusCode::Ok.into())
    } else {
        Ok(StatusCode::Unauthorized.into())
    }
}
