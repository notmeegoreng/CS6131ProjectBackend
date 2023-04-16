use serde::{Deserialize, Serialize};
use sqlx::{Executor, Row};
use tide::{Response, StatusCode};
use crate::models::BasicContainer;
use crate::Request;
use crate::routes::containers::PAGE_SIZE;

#[derive(Debug, Eq, PartialEq, Clone, Deserialize)]
struct CategoryCreate {
    name: String,
    description: String
}

pub async fn category_create(mut req: Request) -> tide::Result {
    if let Some(user_id) = req.session().get::<u32>("user_id") {
        let data: CategoryCreate = req.body_json().await?;
        match sqlx::query!(
            "INSERT INTO audit_log(user_id, log) VALUES (?, ?)",
            user_id, format!("create category `{}`", data.name)
        ).execute(&req.state().db).await {
            Ok(_) => {
                let category_id = sqlx::query!(
                    "INSERT INTO categories(name, description) VALUES(?, ?)",
                    data.name, data.description
                ).execute(&req.state().db).await?.last_insert_id();

                Ok(Response::builder(StatusCode::Created).
                    body(serde_json::to_value(category_id)?)
                    .build())
            },
            Err(sqlx::Error::Database(e))
            if e.code().map_or(false, |s| s == "45000") => {
                Ok(Response::new(StatusCode::Forbidden))
            },
            Err(e) => return Err(tide::Error::new(StatusCode::InternalServerError, e))
        }
    } else {
        Ok(Response::new(StatusCode::Unauthorized))
    }
}

pub async fn category_patch(mut req: Request) -> tide::Result {
    if let Some(user_id) = req.session().get::<u32>("user_id") {
        let category_id: u32 = req.param("category_id")?.parse()?;
        match sqlx::query!(
            "INSERT INTO audit_log(user_id, log) VALUES (?, ?)",
            user_id, format!("Update category with ID `{}`", category_id)
        ).execute(&req.state().db).await {
            Ok(_) => {
                let new: BasicContainer = req.body_json().await?;
                sqlx::query!("UPDATE categories SET name = ?, description = ? WHERE category_id = ?",
                    new.name, new.description, category_id)
                    .execute(&req.state().db).await?;
                Ok(Response::new(StatusCode::NoContent))
            },
            Err(sqlx::Error::Database(e))
            if e.code().map_or(false, |s| s == "45000") => {
                Ok(Response::new(StatusCode::Forbidden))
            },
            Err(e) => return Err(tide::Error::new(StatusCode::InternalServerError, e))
        }
    } else {
        Ok(Response::new(StatusCode::Unauthorized))
    }
}

pub async fn category_delete(req: Request) -> tide::Result {
    if let Some(user_id) = req.session().get::<u32>("user_id") {
        let category_id: u32 = req.param("category_id")?.parse()?;
        match sqlx::query!(
            "INSERT INTO audit_log(user_id, log) VALUES (?, ?)",
            user_id, format!("Delete category with ID `{}`", category_id)
        ).execute(&req.state().db).await {
            Ok(_) => {
                sqlx::query!("DELETE FROM categories WHERE category_id = ?", category_id)
                    .execute(&req.state().db).await?;
                Ok(Response::new(StatusCode::NoContent))
            },
            Err(sqlx::Error::Database(e))
            if e.code().map_or(false, |s| s == "45000") => {
                Ok(Response::new(StatusCode::Forbidden))
            },
            Err(e) => return Err(tide::Error::new(StatusCode::InternalServerError, e))
        }
    } else {
        Ok(Response::new(StatusCode::Unauthorized))
    }
}

#[derive(Debug, Eq, PartialEq, Clone, Deserialize)]
struct ForumCreate {
    category_id: u32,
    name: String,
    description: String
}

pub async fn forum_create(mut req: Request) -> tide::Result {
    if let Some(user_id) = req.session().get::<u32>("user_id") {
        let data: ForumCreate = req.body_json().await?;
        match sqlx::query!(
            "INSERT INTO audit_log(user_id, log) VALUES (?, ?)",
            user_id, format!("Create forum with name `{}`", data.name)
        ).execute(&req.state().db).await {
            Ok(_) => {
                let forum_id = sqlx::query!(
                    "INSERT INTO forums(category_id, name, description) VALUES(?, ?, ?)",
                    data.category_id, data.name, data.description
                ).execute(&req.state().db).await?.last_insert_id();

                Ok(Response::builder(StatusCode::Created).
                    body(serde_json::to_value(forum_id)?)
                    .build())
            },
            Err(sqlx::Error::Database(e))
            if e.code().map_or(false, |s| s == "45000") => {
                Ok(Response::new(StatusCode::Forbidden))
            },
            Err(e) => return Err(tide::Error::new(StatusCode::InternalServerError, e))
        }
    } else {
        Ok(Response::new(StatusCode::Unauthorized))
    }
}

pub async fn forum_patch(mut req: Request) -> tide::Result {
    if let Some(user_id) = req.session().get::<u32>("user_id") {
        let forum_id: u32 = req.param("forum_id")?.parse()?;
        match sqlx::query!(
            "INSERT INTO audit_log(user_id, log) VALUES (?, ?)",
            user_id, format!("Update forum with ID `{}`", forum_id)
        ).execute(&req.state().db).await {
            Ok(_) => {
                let new: BasicContainer = req.body_json().await?;
                sqlx::query!("UPDATE forums SET name = ?, description = ? WHERE forum_id = ?",
                    new.name, new.description, forum_id)
                    .execute(&req.state().db).await?;
                Ok(Response::new(StatusCode::NoContent))
            },
            Err(sqlx::Error::Database(e))
            if e.code().map_or(false, |s| s == "45000") => {
                Ok(Response::new(StatusCode::Forbidden))
            },
            Err(e) => Err(tide::Error::new(StatusCode::InternalServerError, e))
        }
    } else {
        Ok(Response::new(StatusCode::Unauthorized))
    }
}

pub async fn forum_delete(req: Request) -> tide::Result {
    if let Some(user_id) = req.session().get::<u32>("user_id") {
        let forum_id: u32 = req.param("forum_id")?.parse()?;
        match sqlx::query!(
            "INSERT INTO audit_log(user_id, log) VALUES (?, ?)",
            user_id, format!("Delete forum with ID `{}`", forum_id)
        ).execute(&req.state().db).await {
            Ok(_) => {
            sqlx::query!("DELETE FROM forums WHERE forum_id = ?", forum_id)
                .execute(&req.state().db).await?;
            Ok(Response::new(StatusCode::NoContent))
            },
            Err(sqlx::Error::Database(e))
            if e.code().map_or(false, |s| s == "45000") => {
                Ok(Response::new(StatusCode::Forbidden))
            },
            Err(e) => Err(tide::Error::new(StatusCode::InternalServerError, e))
        }
    } else {
        Ok(Response::new(StatusCode::Unauthorized))
    }
}


#[derive(Debug, Eq, PartialEq, Clone, Deserialize)]
struct TopicCreate {
    forum_id: u32,
    name: String,
    description: String
}

pub async fn topic_create(mut req: Request) -> tide::Result {
    if let Some(user_id) = req.session().get::<u32>("user_id") {
        let data: TopicCreate = req.body_json().await?;
        match sqlx::query!(
            "INSERT INTO audit_log(user_id, log) VALUES (?, ?)",
            user_id, format!("Create topic named `{}`", data.name)
        ).execute(&req.state().db).await {
            Ok(_) => {
                let topic_id = sqlx::query!(
                    "INSERT INTO topics(forum_id, name, description) VALUES(?, ?, ?)",
                    data.forum_id, data.name, data.description
                ).execute(&req.state().db).await?.last_insert_id();

                Ok(Response::builder(StatusCode::Created).
                    body(serde_json::to_value(topic_id)?)
                    .build())
            },
            Err(sqlx::Error::Database(e))
            if e.code().map_or(false, |s| s == "45000") => {
                Ok(Response::new(StatusCode::Forbidden))
            },
            Err(e) => Err(tide::Error::new(StatusCode::InternalServerError, e))
        }
    } else {
        Ok(Response::new(StatusCode::Unauthorized))
    }
}

pub async fn topic_patch(mut req: Request) -> tide::Result {
    if let Some(user_id) = req.session().get::<u32>("user_id") {
        let topic_id: u32 = req.param("topic_id")?.parse()?;
        match sqlx::query!(
            "INSERT INTO audit_log(user_id, log) VALUES (?, ?)",
            user_id, format!("Update topic with ID `{}`", topic_id)
        ).execute(&req.state().db).await {
            Ok(_) => {
                let new: BasicContainer = req.body_json().await?;
                sqlx::query!("UPDATE topics SET name = ?, description = ? WHERE topic_id = ?",
                new.name, new.description, topic_id)
                    .execute(&req.state().db).await?;
                Ok(Response::new(StatusCode::NoContent))
            },
            Err(sqlx::Error::Database(e))
            if e.code().map_or(false, |s| s == "45000") => {
                Ok(Response::new(StatusCode::Forbidden))
            }
            Err(e) => return Err(tide::Error::new(StatusCode::InternalServerError, e))
        }
    } else {
        Ok(Response::new(StatusCode::Unauthorized))
    }
}

pub async fn topic_delete(req: Request) -> tide::Result {
    if let Some(user_id) = req.session().get::<u32>("user_id") {
        let topic_id: u32 = req.param("topic_id")?.parse()?;
        match sqlx::query!(
            "INSERT INTO audit_log(user_id, log) VALUES (?, ?)",
            user_id, format!("Delete topic with ID `{}`", topic_id)
        ).execute(&req.state().db).await {
            Ok(_) => {
                sqlx::query!("DELETE FROM topics WHERE topic_id = ?", topic_id)
                    .execute(&req.state().db).await?;
                Ok(Response::new(StatusCode::NoContent))
            },
            Err(sqlx::Error::Database(e))
            if e.code().map_or(false, |s| s == "45000") => {
                Ok(Response::new(StatusCode::Forbidden))
            }
            Err(e) => return Err(tide::Error::new(StatusCode::InternalServerError, e))
        }
    } else {
        Ok(Response::new(StatusCode::Unauthorized))
    }
}


#[derive(Debug, Eq, PartialEq, Clone, Deserialize)]
struct ThreadCreate {
    topic_id: u32,
    name: String,
    post_content: String
}

pub async fn thread_create(mut req: Request) -> tide::Result {
    if let Some(user_id) = req.session().get::<u32>("user_id") {
        let data: ThreadCreate = req.body_json().await?;
        let mut tx = req.state().db.begin().await?;
        let t_result = sqlx::query!(
            "INSERT INTO threads(name, topic_id) VALUES(?, ?)",
            data.name, data.topic_id
        ).execute(&mut tx).await?;
        let thread_id = t_result.last_insert_id() as u32;
        sqlx::query!(
            "INSERT INTO posts(thread_id, user_id, content) VALUES (?, ?, ?)",
            thread_id, user_id, data.post_content
        ).execute(&mut tx).await?;
        tx.commit().await?;
        Ok(Response::builder(StatusCode::Created)
            .header(
                "Location",
                req.url().join(&thread_id.to_string())?.as_str()
            ).body(serde_json::to_value(thread_id)?).build())
    } else {
        Ok(Response::new(StatusCode::Unauthorized))
    }
}

pub async fn thread_delete(req: Request) -> tide::Result {
    if let Some(user_id) = req.session().get::<u32>("user_id") {
        let thread_id: u32 = req.param("thread_id")?.parse()?;
        let allowed = sqlx::query!(
            "SELECT (user_id = ? OR (SELECT is_admin FROM users u WHERE u.user_id = p.user_id)) `ok!: bool`
             FROM posts p WHERE post_pos = 1 AND thread_id = ?",
            user_id, thread_id
        ).fetch_one(&req.state().db).await?;
        if allowed.ok {
            sqlx::query!("DELETE FROM threads WHERE thread_id = ?", thread_id)
                .execute(&req.state().db).await?;
            Ok(Response::new(StatusCode::NoContent))
        } else {
            Ok(Response::new(StatusCode::Forbidden))
        }
    } else {
        Ok(Response::new(StatusCode::Unauthorized))
    }
}

#[derive(Deserialize)]
struct PostCreate {
    thread_id: u32,
    content: String
}

#[derive(Serialize)]
struct PostCreated {
    post_id: u32,
    page_num: u32
}

pub async fn post_create(mut req: Request) -> tide::Result {
    if let Some(user_id) = req.session().get::<u32>("user_id") {
        let data: PostCreate = req.body_json().await?;
        let r = sqlx::query!(
            "CALL insert_post(?, ?, ?)",
            data.thread_id, user_id, data.content
        ).fetch_one(&req.state().db).await?;
        tide::log::debug!("post create record: {:?}", r);
        let resp = Response::builder(StatusCode::Created)
            .body(serde_json::to_value(PostCreated {
                post_id: r.get_unchecked(0),
                page_num: (r.get_unchecked::<u32, _>(1) - 1) / PAGE_SIZE as u32 + 1
            })?)
            .build();
        tide::log::debug!("post create resp: {:?}", resp);
        Ok(resp)
    } else {
        Ok(Response::new(StatusCode::Unauthorized))
    }
}

pub async fn post_delete(req: Request) -> tide::Result {
    if let Some(user_id) = req.session().get::<u32>("user_id") {
        let post_id: u32 = req.param("post_id")?.parse()?;
        let perm_check = sqlx::query!(
            "SELECT (
               EXISTS(SELECT * FROM posts p WHERE p.user_id = u.user_id AND post_id = ?)
            ) `poster: bool`, is_admin `admin: bool`
            FROM users u WHERE u.user_id = ?",
            post_id, user_id
        ).fetch_one(&req.state().db).await?;
        if perm_check.poster || perm_check.admin {
            let mut tx = req.state().db.begin().await?;
            // trigger dont work because mysql cannot update current table
            // procedure dont work because variables must be scalars and i cannot split a query with 2 cols
            let info = sqlx::query!(
                "SELECT thread_id, post_pos, last_pos FROM posts INNER JOIN threads USING (thread_id)\
                 WHERE post_id = ?", post_id
            ).fetch_one(&mut tx).await?;
            tide::log::debug!("DELETE POST: {}, {}", info.thread_id, info.post_pos);
            if info.post_pos == 1 {
                sqlx::query!("DELETE FROM threads WHERE thread_id = ?", info.thread_id)
                    .execute(&mut tx).await?;
                sqlx::query!(
                    "INSERT INTO audit_log(user_id, log) VALUES (?, ?)",
                    user_id, format!("Deleted thread (ID: {})", info.thread_id)
                ).execute(&mut tx).await?;
            } else {
                // i do not like mysql.
                tx.execute("DROP TRIGGER IF EXISTS post_delete_up_last_pos");
                sqlx::query!("UPDATE threads SET last_pos = NULL WHERE thread_id = ?", info.thread_id)
                    .execute(&mut tx).await?;
                sqlx::query!("DELETE FROM posts WHERE post_id = ?", post_id)
                    .execute(&mut tx).await?;
                tide::log::debug!("DELETE POST: {:?}",
                    sqlx::query!("SELECT EXISTS(SELECT * FROM threads WHERE thread_id = ?) `a: bool`",
                        info.thread_id).fetch_one(&mut tx).await?.a);
                sqlx::query!(
                    "UPDATE posts SET post_pos = post_pos - 1 WHERE thread_id = ? AND post_pos > ?",
                    info.thread_id, info.post_pos
                ).execute(&mut tx).await?;
                sqlx::query!("UPDATE threads SET last_pos = ? - 1 WHERE thread_id = ?", info.last_pos, info.thread_id)
                    .execute(&mut tx).await?;
                if !perm_check.poster {
                    sqlx::query!(
                        "INSERT INTO audit_log(user_id, log) VALUES (?, ?)",
                        user_id, format!("Deleted post (Post ID: {}, Thread ID: {})", post_id, info.thread_id)
                    ).execute(&mut tx).await?;
                }
            }

            tx.commit().await?;
            Ok(Response::new(StatusCode::ResetContent))
        } else {
            Ok(Response::new(StatusCode::Forbidden))
        }
    } else {
        Ok(Response::new(StatusCode::Unauthorized))
    }
}
