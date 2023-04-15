use std::collections::{HashMap, hash_map::Entry};

use tide::{Response, StatusCode};
use async_std::stream::StreamExt;
use tide::convert::Serialize;

use crate::Request;
use crate::models::{Container, ContainerData, BasicContainer, User};
use crate::routes::containers::PAGE_SIZE;
use crate::utils::{data_into_hashmap, route_search, SearchQuery};


route_search!(
    forum_search,
    "SELECT category_id p_id, c.name p_name, c.description p_descr,
     forum_id c_id, f.name c_name, f.description c_descr
     FROM categories c INNER JOIN forums f USING (category_id)
     WHERE f.name LIKE CONCAT('%', ?, '%')"
);

route_search!(
    topic_search,
    "SELECT forum_id p_id, f.name p_name, f.description p_descr,
     topic_id c_id, t.name c_name, t.description c_descr
     FROM forums f INNER JOIN topics t USING (forum_id)
     WHERE t.name LIKE CONCAT('%', ?, '%')"
);


#[derive(Serialize)]
struct ThreadAllInfo {
    id: u32,
    name: String,
    description: String,
    user_id: u32,
    username: String,
    is_avatar_set: bool
}


pub async fn thread_search(req: Request) -> tide::Result {
    let query = req.query::<SearchQuery>()?;

    if let Some ( q ) = query . q {
        let mut s = sqlx::query!(
            "SELECT topic_id p_id, top.name p_name, top.description p_descr,
             t.thread_id c_id, t.name c_name, p.content c_descr,
             user_id, username, is_avatar_set `is_avatar_set: bool`
             FROM topics top INNER JOIN threads t USING (topic_id)
             INNER JOIN posts p ON (t.thread_id = p.thread_id AND post_pos = 1)
             INNER JOIN users USING (user_id)
             WHERE t.name LIKE CONCAT('%', ?, '%')", q
        ).fetch(&req.state().db);
        let mut data: HashMap<u32, ContainerData<BasicContainer, ThreadAllInfo>> = HashMap::new();
        while let Some(Ok(r)) = s.next().await {
            match data.entry(r.p_id) {
                Entry::Occupied(mut e) => {
                    e.get_mut().children.push(ThreadAllInfo {
                        id: r.c_id, name: r.c_name, description: r.c_descr,
                        user_id: r.user_id, username: r.username, is_avatar_set: r.is_avatar_set
                    });
                },
                Entry::Vacant(e) => {
                    e.insert(ContainerData {
                        container: BasicContainer { name: r.p_name, description: r.p_descr },
                        children: vec!(ThreadAllInfo {
                            id: r.c_id, name: r.c_name, description: r.c_descr,
                            user_id: r.user_id, username: r.username, is_avatar_set: r.is_avatar_set
                        })
                    });
                }
            }
        }
        return Ok(serde_json::to_value(data)?.into());
    }
    Ok(Response::builder(StatusCode::BadRequest)
        .content_type("text/plain")
        .body("Please provide a query `q`")
        .build())
}

#[derive(Serialize)]
struct PostSpecific {
    post_id: u32,
    content: String,
    page_num: u32,
    user_id: u32,
    username: String,
    is_avatar_set: bool,
}

pub async fn post_search(req: Request) -> tide::Result {
    let query = req.query::<SearchQuery>()?;
    if let Some(q) = query.q {
        let mut s = sqlx::query!(
            "SELECT t.thread_id p_id, t.name p_name, pf.content p_descr,
             p.post_id post_id, p.user_id user_id, p.content content, p.post_pos,
             username, is_avatar_set `is_avatar_set: bool`
             FROM threads t INNER JOIN posts pf ON (t.thread_id = pf.thread_id AND post_pos = 1)
             INNER JOIN posts p ON (t.thread_id = p.thread_id) INNER JOIN users u ON (u.user_id = p.user_id)
             WHERE p.content LIKE CONCAT('%', ?, '%') ORDER BY p.thread_id, p.post_pos", q
        ).fetch(& req.state().db);
        let mut data: HashMap<u32, ContainerData<BasicContainer, PostSpecific>> = HashMap::new();
        while let Some(Ok(r)) = s.next().await {
            match data.entry(r.p_id) {
                Entry::Occupied(mut e) => {
                    e.get_mut().children.push(PostSpecific {
                        post_id: r.post_id, content: r.content, page_num: r.post_pos / PAGE_SIZE as u32 + 1,
                        user_id: r.user_id, username: r.username, is_avatar_set: r.is_avatar_set
                    });
                },
                Entry::Vacant(e) => {
                    e.insert(ContainerData {
                        container: BasicContainer { name: r.p_name, description: r.p_descr },
                        children: vec!(PostSpecific {
                            post_id: r.post_id, content: r.content, page_num: r.post_pos / PAGE_SIZE as u32 + 1,
                            user_id: r.user_id, username: r.username, is_avatar_set: r.is_avatar_set
                        })
                    });
                }
            }
        }
        return Ok(serde_json::to_value(data)?.into());
    }
    Ok(Response::builder(StatusCode::BadRequest)
        .content_type("text/plain")
        .body("Please provide a query `q`")
        .build())
}


pub async fn user_search(req: Request) -> tide::Result {
    let query = req.query::<SearchQuery>()?;
    let result;
    if let Some(q) = query.q {
        result = sqlx::query_as!(
            User,
            "SELECT user_id, username, profile_tag, description,
             is_avatar_set AS `is_avatar_set: _`, is_admin AS `is_admin: _`
             FROM users WHERE username LIKE CONCAT('%', ?, '%')",
            q
        ).fetch_all(&req.state().db).await?;
        return Ok(serde_json::to_value(result)?.into());
    }
    Ok(Response::builder(StatusCode::BadRequest)
        .content_type("text/plain")
        .body("Please provide a query `q`")
        .build())
}

