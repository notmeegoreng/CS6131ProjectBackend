use async_session::Session;
use tide::StatusCode;
use serde::{Serialize, Deserialize};
use tide::Response;

use crate::{Request, utils::{wrap_error, auth}};
use crate::utils::sessions::SessionWorkaroundExt;

const PRE_AUTH_KEY: &str = "pre_auth";

#[derive(Debug, Eq, PartialEq, Clone, Serialize, Deserialize)]
struct Login {
    username: String,
    password: String
}

pub async fn pre_auth(mut req: Request) -> tide::Result {
    let sess = req.session_mut();
    sess.insert_raw(PRE_AUTH_KEY, "_".to_string());
    Ok("".into())
}

pub fn check_pre_auth(sess: &mut Session) -> Option<Response> {
    if let Some(_) = sess.get_raw(PRE_AUTH_KEY) {
        return None;
    }
    Some(Response::builder(StatusCode::Unauthorized).body("pre auth required").build())
}

pub async fn register(mut req: Request) -> tide::Result {
    if let Some(resp) = check_pre_auth(req.session_mut()) {
        return Ok(resp)
    }

    let reg_data: Login = req.body_json().await?;
    let hash_data = wrap_error!(auth::hash(&reg_data.password), StatusCode::InternalServerError);
    let result = sqlx::query!(
        "INSERT INTO users(username, credentials, salt) VALUES (?, ?, ?)",
        reg_data.username,
        &hash_data.hash[..],
        &hash_data.salt[..]
    ).execute(&req.state().db).await?;
    let user_id = result.last_insert_id();
    let sess = req.session_mut(); // must reborrow here so it can be dropped earlier
    sess.mark_for_regenerate();
    sess.insert("user_id", user_id)?;
    sess.remove(PRE_AUTH_KEY);
    Ok(Response::builder(StatusCode::Created).body(user_id.to_string()).build())
}

pub async fn login<'a>(mut req: Request) -> tide::Result {
    if let Some(resp) = check_pre_auth(req.session_mut()) {
        return Ok(resp)
    }

    let login_data: Login = req.body_json().await?;
    let data = match sqlx::query!(
        "SELECT user_id, credentials AS `creds: Vec<u8>`, salt as `salt: Vec<u8>` FROM users WHERE username = ?",
        &login_data.username
    ).fetch_optional(&req.state().db).await? {
        None => return Ok(Response::builder(StatusCode::Forbidden).body("username not found").build()),
        Some(d) => d
    };

    match auth::verify(
        login_data.password,
        auth::Hashed::new_check_length(data.creds.as_slice(), data.salt.as_slice())
    ) {
        Ok(()) => (),
        Err(auth::PASSWORD_ERROR) => return Ok(
            Response::builder(StatusCode::Forbidden).body("incorrect password").build()),
        Err(e) => return Err(tide::Error::new(StatusCode::InternalServerError, e))
    }
    let sess = req.session_mut();
    sess.mark_for_regenerate();
    sess.insert("user_id", data.user_id)?;
    sess.remove(PRE_AUTH_KEY);
    Ok(data.user_id.to_string().into())
}

pub async fn logout(mut req: Request) -> tide::Result {
    req.session_mut().destroy();
    Ok("".into())
}
