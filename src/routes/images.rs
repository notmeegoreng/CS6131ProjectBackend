use async_std::{path::{Path, PathBuf}, fs};
use lazy_static::lazy_static;
use serde::{Serialize, Deserialize};
use tide::StatusCode;
use crate::Request;

#[derive(Serialize, Deserialize)]
struct Image {
    name: String,
    data: Vec<u8>
}

lazy_static! {
    static ref IMAGE_DIR: &'static Path = Path::new(r#".\images"#);
    static ref AVATAR_DIR: PathBuf = IMAGE_DIR.join("avatars");
}

pub(crate) async fn set_avatar(mut req: Request) -> tide::Result {
    if let Some(user_id) = req.session().get::<u32>("user_id") {
        let file_type = req.content_type()
            .ok_or(tide::Error::from_str(StatusCode::BadRequest, "Content-Type Invalid"))?;
        if !matches!(file_type.subtype(), "jpeg" | "png") {
            return Err(tide::Error::from_str(StatusCode::BadRequest, "Content-Type Not Accepted"));
        }
        fs::write(
            AVATAR_DIR.join(format!("{}", user_id)),
            req.body_bytes().await?
        ).await?;
        sqlx::query!("UPDATE users SET is_avatar_set = TRUE WHERE user_id = ?", user_id)
            .execute(&req.state().db).await?;

        Ok(StatusCode::NoContent.into())
    } else {
        Ok(StatusCode::Unauthorized.into())
    }
}
