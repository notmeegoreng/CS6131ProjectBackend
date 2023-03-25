use serde::Deserialize;
use tide::{Response, StatusCode};
use crate::Request;

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

                Ok(Response::builder(StatusCode::Created)
                    .header(
                        "Location",
                        req.url().join(&category_id.to_string())?.as_str()
                    ).build()
                )
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
            user_id, format!("create forum `{}`", data.name)
        ).execute(&req.state().db).await {
            Ok(_) => {
                let forum_id = sqlx::query!(
                    "INSERT INTO forums(category_id, name, description) VALUES(?, ?, ?)",
                    data.category_id, data.name, data.description
                ).execute(&req.state().db).await?.last_insert_id();

                Ok(Response::builder(StatusCode::Created)
                    .header(
                        "Location",
                        req.url().join(&forum_id.to_string())?.as_str()
                    ).build()
                )
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
            user_id, format!("create topic `{}`", data.name)
        ).execute(&req.state().db).await {
            Ok(_) => {
                let topic_id = sqlx::query!(
                    "INSERT INTO topics(forum_id, name, description) VALUES(?, ?, ?)",
                    data.forum_id, data.name, data.description
                ).execute(&req.state().db).await?.last_insert_id();

                Ok(Response::builder(StatusCode::Created)
                    .header(
                        "Location",
                        req.url().join(&topic_id.to_string())?.as_str()
                    ).build()
                )
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
struct ThreadCreate {
    topic_id: u32,
    name: String,
    description: Option<String>,
    post_content: String
}

pub async fn thread_create(mut req: Request) -> tide::Result {
    if let Some(user_id) = req.session().get::<u32>("user_id") {
        let data: ThreadCreate = req.body_json().await?;
        let mut tx = req.state().db.begin().await?;
        let t_result = sqlx::query!(
                "INSERT INTO threads(name, description, topic_id) VALUES(?, ?, ?)",
                data.name, data.description, data.topic_id
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
            ).build())
    } else {
        Ok(Response::new(StatusCode::Unauthorized))
    }
}

#[derive(Debug, Eq, PartialEq, Clone, Deserialize)]
struct PostCreate {
    thread_id: u32,
    content: String
}

pub async fn post_create(mut req: Request) -> tide::Result {
    if let Some(user_id) = req.session().get::<u32>("user_id") {
        let data: PostCreate = req.body_json().await?;
        let t_result = sqlx::query!(
                "INSERT INTO posts(user_id, thread_id, content) VALUES(?, ?, ?)",
                user_id, data.thread_id, data.content
            ).execute(&req.state().db).await?;
        let post_id = t_result.last_insert_id() as u32;
        Ok(Response::builder(StatusCode::Created)
            .header(
                "Location",
                req.url().join(&post_id.to_string())?.as_str()
            ).build())
    } else {
        Ok(Response::new(StatusCode::Unauthorized))
    }
}
