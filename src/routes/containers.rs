use std::collections::{HashMap, hash_map::Entry};
use serde::Serialize;
use tide::StatusCode;
use async_std::stream::StreamExt;

use crate::{Request, utils::route_get};
use crate::models::*;

#[derive(Serialize)]
struct ThreadWID {
    thread_id: u32,
    last_pos: u32,
    name: String,
}

#[derive(Serialize)]
struct Thread {
    last_pos: u32,
    name: String,
}

#[derive(Serialize)]
struct Post {
    post_id: u32,
    user_id: u32,
    content: String,
}

route_get!(
    home, req, {
        let mut s = sqlx::query!(
            "SELECT category_id c_id, c.name c_name, c.description c_descr,
            f.forum_id f_id, f.name f_name, f.description f_descr
            FROM categories c INNER JOIN forums f USING (category_id)").fetch(&req.state().db);

        let mut data: HashMap::<u32, ContainerData<0>> = HashMap::new();
        while let Some(Ok(r)) = s.next().await {
            match data.entry(r.c_id) {
                Entry::Occupied(mut e) => {
                    e.get_mut().children.push(Container {
                        id: r.f_id, name: r.f_name, description: r.f_descr
                    });
                },
                Entry::Vacant(e) => {
                    e.insert(ContainerData {
                        parents: [],
                        container: BasicContainer {name: r.c_name, description: r.c_descr},
                        children: vec!(Container {
                            id: r.f_id, name: r.f_name, description: r.f_descr
                        })
                    });
                }
            }
        }
        data
    }
);

pub async fn forum_data(req: Request) -> tide::Result {
    let forum_id = req.param("forum_id")?.parse::<u32>()?;

    if let Some(r) = sqlx::query!(
        "SELECT category_id AS c_id, c.name AS `c_name!`, f.name AS f_name, f.description AS f_description
         FROM forums AS f LEFT JOIN categories AS c USING (category_id) WHERE forum_id = ?",
        forum_id
    ).fetch_optional(&req.state().db).await? {
        let vec = sqlx::query_as!(Container,
            "SELECT topic_id AS id, name, description FROM topics WHERE forum_id = ?",
            forum_id
        ).fetch_all(&req.state().db).await?;
        return Ok(serde_json::to_value(ContainerData {
            parents: [IDContainer { id: r.c_id, name: r.c_name }],
            container: BasicContainer { name: r.f_name, description: r.f_description },
            children: vec
        })?.into());
    }
    return Ok(StatusCode::NotFound.into());
}

const PAGE_SIZE: u16 = 10;

#[derive(Serialize)]
struct TopicInfo {
    parents: [IDContainer; 2],
    container: BasicContainer
}

pub async fn topic_info(req: Request) -> tide::Result {
    let topic_id = req.param("topic_id")?.parse::<u32>()?;
    if let Some(r) = sqlx::query!(
        "SELECT category_id AS `c_id!`, c.name AS `c_name!`, forum_id AS f_id, f.name AS `f_name!`,
         t.name AS t_name, t.description
         FROM topics AS t
         LEFT JOIN forums AS f USING (forum_id)
         LEFT JOIN categories AS c USING (category_id)
         WHERE topic_id = ?",
        topic_id
    ).fetch_optional(&req.state().db).await? {
        return Ok(serde_json::to_value(TopicInfo {
            parents: [
                IDContainer { id: r.c_id, name: r.c_name },
                IDContainer { id: r.f_id, name: r.f_name }
            ],
            container: BasicContainer { name: r.t_name, description: r.description }
        })?.into());
    }
    Ok(StatusCode::NotFound.into())
}

pub async fn topic_pages(req: Request) -> tide::Result {
    let topic_id = req.param("topic_id")?.parse::<u32>()?;
    let vec = sqlx::query_as!(
        IDContainer,
        "SELECT t.thread_id AS id, name FROM threads AS t WHERE topic_id = ? ORDER BY time LIMIT ? OFFSET ?",
        topic_id, PAGE_SIZE, PAGE_SIZE * (req.param("page_num")?.parse::<u16>()? - 1)
    ).fetch_all(&req.state().db).await?;

    if vec.len() != 0 || sqlx::query!(
        "SELECT 1 AS ex FROM topics WHERE topic_id = ?",
        topic_id
    ).fetch_optional(&req.state().db).await?.is_some() {
        return Ok(serde_json::to_value(vec)?.into());
    }
    Ok(StatusCode::NotFound.into())
}

#[derive(Serialize)]
struct ThreadInfo {
    parents: [IDContainer; 3],
    container: Thread
}

pub async fn thread_info(req: Request) -> tide::Result {
    let thread_id = req.param("thread_id")?.parse::<u32>()?;
    if let Some(r) = sqlx::query!(
        "SELECT category_id AS `c_id!`, c.name AS `c_name!`,
         forum_id AS `f_id!`, f.name AS `f_name!`,
         topic_id AS t_id, t.name AS `t_name!`,
         th.name AS th_name, COALESCE(th.last_pos, 0) AS `last_pos: u32`
         FROM threads AS th
         LEFT JOIN topics AS t USING (topic_id)
         LEFT JOIN forums AS f USING (forum_id)
         LEFT JOIN categories AS c USING (category_id)
         WHERE thread_id = ?", thread_id
    ).fetch_optional(&req.state().db).await? {
        return Ok(serde_json::to_value(ThreadInfo {
            parents: [
                IDContainer { id: r.c_id, name: r.c_name },
                IDContainer { id: r.f_id, name: r.f_name },
                IDContainer { id: r.t_id, name: r.t_name }
            ],
            container: Thread {
                last_pos: r.last_pos,
                name: r.th_name,
            }
        })?.into())
    }
    Ok(StatusCode::NotFound.into())
}

pub async fn thread_pages(req: Request) -> tide::Result {
    let thread_id = req.param("thread_id")?.parse::<u32>()?;
    let vec = sqlx::query_as!(
        Post,
        "SELECT post_id, user_id, content FROM posts WHERE thread_id = ? ORDER BY post_pos LIMIT ? OFFSET ?",
        thread_id, PAGE_SIZE, PAGE_SIZE * (req.param("page_num")?.parse::<u16>()? - 1)
    ).fetch_all(&req.state().db).await?;

    if vec.len() != 0 || sqlx::query!(
        "SELECT 1 AS ex FROM threads WHERE thread_id = ?",
        thread_id
    ).fetch_optional(&req.state().db).await?.is_some() {
        return Ok(serde_json::to_value(vec)?.into());
    }
    Ok(StatusCode::NotFound.into())
}
