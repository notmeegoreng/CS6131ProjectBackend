use std::collections::{HashMap, hash_map::Entry};
use serde::Serialize;
use tide::{Response, StatusCode};
use async_std::stream::StreamExt;

use crate::{Request, utils::{data_into_hashmap, route_get}};
use crate::models::*;

#[derive(Serialize)]
struct ThreadWID {
    thread_id: u32,
    last_pos: u32,
    name: String,
}

#[derive(Serialize)]
struct Thread {
    id: u32,
    user_id: u32,
    name: String,
    description: String
}

#[derive(Serialize)]
pub struct BasicUser {
    pub username: String,
    pub is_avatar_set: bool
}

route_get!(
    home, req, {
        let mut s = sqlx::query!(
            "SELECT category_id p_id, c.name p_name, c.description p_descr,
            f.forum_id c_id, f.name c_name, f.description c_descr
            FROM categories c INNER JOIN forums f USING (category_id)").fetch(&req.state().db);

        data_into_hashmap!(s)
    }
);

#[derive(Serialize)]
struct PostWPos {
    post_id: u32,
    user_id: u32,
    content: String,
    post_pos: u32
}

#[derive(Serialize)]
struct LatestData {
    threads: Vec<ContainerData<Container, PostWPos>>,
    users: HashMap<u32, BasicUser>
}

pub async fn latest_posts(req: Request) -> tide::Result {
    tide::log::debug!("latest_posts_called");
    let mut s = sqlx::query!(
        "WITH ts AS (
            SELECT t.thread_id, name, last_pos, lp.time FROM threads t
            INNER JOIN posts lp ON (t.thread_id = lp.thread_id AND t.last_pos = lp.post_pos)
            ORDER BY lp.time DESC LIMIT 10
        ) SELECT thread_id, name, pf.content description, p.post_id, p.user_id, p.content, p.post_pos,
        username, profile_tag, is_avatar_set AS `is_avatar_set: bool`, is_admin AS `is_admin: bool`
        FROM ts INNER JOIN posts p USING (thread_id) INNER JOIN posts pf USING (thread_id)
        INNER JOIN users u ON (p.user_id = u.user_id)
        WHERE pf.post_pos = 1 AND p.post_pos > IF(last_pos <= 5, 0, last_pos - 5)
        ORDER BY ts.time DESC, p.post_pos"
    ).fetch(&req.state().db);

    let mut threads = vec!();
    let mut users = HashMap::new();
    let mut thread;

    macro_rules! make_thread {
        ($r:expr) => {
            ContainerData {
                container: Container { id: $r.thread_id, name: $r.name, description: $r.description },
                children: vec!(PostWPos {
                    post_id: $r.post_id, user_id: $r.user_id,
                    content: $r.content, post_pos: $r.post_pos
                })
            }
        };
    }
    let next = s.next().await;
    if let Some(Ok(r)) = next {
        thread = make_thread!(r);
        users.insert(r.user_id, BasicUser {
            username: r.username,
            is_avatar_set: r.is_avatar_set
        });
    } else {
        return if let Some(Err(e)) = next {
            Err(tide::Error::new(StatusCode::InternalServerError, e))
        } else {
            Ok(Response::new(StatusCode::InternalServerError))
        }
    }

    while let Some(Ok(r)) = s.next().await {
        if r.thread_id == thread.container.id {
            thread.children.push(PostWPos {
                post_id: r.post_id, user_id: r.user_id,
                content: r.content, post_pos: r.post_pos
            });
        } else {
            threads.push(thread);
            thread = make_thread!(r);
        }
        if let Entry::Vacant(e) = users.entry(r.user_id) {
            e.insert(BasicUser {
                username: r.username,
                is_avatar_set: r.is_avatar_set
            });
        }
    }
    threads.push(thread);
    Ok(serde_json::to_value(LatestData { threads, users })?.into())
}


route_get!(
    all_categories, req,  {
        sqlx::query_as!(
            IDContainer,
            "SELECT category_id id, name FROM categories ORDER BY category_id"
        ).fetch_all(&req.state().db).await?
    }
);

route_get!(
    all_forums, req,  {
        sqlx::query_as!(
            IDContainer,
            "SELECT forum_id id, name FROM forums ORDER BY category_id, forum_id"
        ).fetch_all(&req.state().db).await?
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
        return Ok(serde_json::to_value(ContainerDataParents {
            parents: [IDContainer { id: r.c_id, name: r.c_name }],
            container: BasicContainer { name: r.f_name, description: r.f_description },
            children: vec
        })?.into());
    }
    return Ok(StatusCode::NotFound.into());
}

route_get!(
    all_topics, req,  {
        sqlx::query_as!(
            IDContainer,
            "SELECT topic_id id, name FROM topics ORDER BY forum_id, topic_id"
        ).fetch_all(&req.state().db).await?
    }
);

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

pub const PAGE_SIZE: u16 = 10;

#[derive(Serialize)]
struct TopicData {
    children: Vec<Thread>,
    users: HashMap<u32, BasicUser>
}

pub async fn topic_pages(req: Request) -> tide::Result {
    let topic_id = req.param("topic_id")?.parse::<u32>()?;
    let vec = sqlx::query!(
        "SELECT t.thread_id AS id, name, user_id, username, content, is_avatar_set AS `is_avatar_set: bool`
         FROM threads AS t INNER JOIN posts USING (thread_id) INNER JOIN users USING (user_id)
         WHERE topic_id = ? AND post_pos = 1 ORDER BY t.time LIMIT ? OFFSET ?",
        topic_id, PAGE_SIZE, PAGE_SIZE * (req.param("page_num")?.parse::<u16>()? - 1)
    ).fetch_all(&req.state().db).await?;

    if vec.len() != 0 || sqlx::query!(
        "SELECT 1 AS ex FROM topics WHERE topic_id = ?",
        topic_id
    ).fetch_optional(&req.state().db).await?.is_some() {
        let mut children = vec![];
        let mut users = HashMap::new();
        for r in vec {
            children.push(Thread {
                id: r.id, name: r.name, user_id: r.user_id, description: r.content
            });
            if let Entry::Vacant(e) = users.entry(r.user_id) {
                e.insert(BasicUser {
                    username: r.username,
                    is_avatar_set: r.is_avatar_set
                });
            }
        }
        return Ok(serde_json::to_value(
            TopicData {
                children,
                users
            }
        )?.into())
    }
    Ok(StatusCode::NotFound.into())
}

#[derive(Serialize)]
struct ThreadI {
    last_pos: u32,
    name: String
}

#[derive(Serialize)]
struct ThreadInfo {
    parents: [IDContainer; 3],
    container: ThreadI
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
            container: ThreadI {
                last_pos: r.last_pos,
                name: r.th_name,
            }
        })?.into())
    }
    Ok(StatusCode::NotFound.into())
}

#[derive(Serialize, Debug)]
pub struct Reaction {
    count: u32,
    reacted: bool
}

#[derive(Serialize, Debug)]
struct Post {
    post_id: u32,
    user_id: u32,
    content: String,
    reactions: HashMap<String, Reaction>
}

#[derive(Serialize)]
struct PostUser {
    user_id: u32,
    username: String,
    profile_tag: String,
    is_admin: bool,
    is_avatar_set: bool
}

#[derive(Serialize)]
struct ThreadData {
    posts: Vec<Post>,
    users: HashMap<u32, PostUser>
}

pub async fn thread_pages(req: Request) -> tide::Result {
    let thread_id = req.param("thread_id")?.parse::<u32>()?;
    // there is no user with id 0
    let user_id = req.session().get::<u32>("user_id").unwrap_or(0);
    let vec = sqlx::query!("
        WITH p AS (
            SELECT post_pos, post_id, user_id, content, username, profile_tag, is_avatar_set, is_admin
            FROM posts INNER JOIN users USING (user_id)
            WHERE thread_id = ? ORDER BY post_pos LIMIT ? OFFSET ?
        )
        SELECT post_id, user_id, content, username, profile_tag, reaction, r_count `r_count: u32`,
        is_avatar_set `is_avatar_set: bool`, is_admin `is_admin: bool`, reacted `reacted: bool`
        FROM p LEFT JOIN
        (
            SELECT post_id, reaction, time, COUNT(*) r_count, MAX(reactor_id = ?) reacted
            FROM reactions_user ru INNER JOIN reactions_time rt USING (post_id, reaction)
            GROUP BY post_id, reaction
        ) r USING (post_id)
        ORDER BY post_pos, r.time",
        thread_id, PAGE_SIZE, PAGE_SIZE * (req.param("page_num")?.parse::<u16>()? - 1), user_id
    ).fetch_all(&req.state().db).await?;

    if vec.len() != 0 {
        let mut posts = vec![];
        let mut users = HashMap::new();
        let mut it = vec.into_iter();
        let r = it.next().unwrap(); // len != 0 already checked
        let mut current = Post {
            post_id: r.post_id,
            user_id: r.user_id,
            content: r.content,
            reactions: HashMap::new()
        };
        users.insert(r.user_id, PostUser {
            user_id: r.user_id,
            username: r.username,
            profile_tag: r.profile_tag,
            is_avatar_set: r.is_avatar_set,
            is_admin: r.is_admin
        });
        if let Some(react) = r.reaction {
            // since `react` exists, `r_count` and `reacted` do too
            current.reactions.insert(react, Reaction { count: r.r_count.unwrap(), reacted: r.reacted.unwrap() } );
        }

        for r in it {
            if current.post_id == r.post_id {
                if let Some(react) = r.reaction {
                    current.reactions.insert(react,Reaction { count: r.r_count.unwrap(), reacted: r.reacted.unwrap() });
                }
            } else {
                posts.push(current);
                current = Post {
                    post_id: r.post_id,
                    user_id: r.user_id,
                    content: r.content,
                    reactions: HashMap::new()
                };
                if let Some(react) = r.reaction {
                    // `react` is defined so `r_count` and `reacted` are
                    current.reactions.insert(react, Reaction {
                        count: r.r_count.unwrap(), reacted: r.reacted.unwrap()
                    });
                }
            }
            if let Entry::Vacant(e) = users.entry(r.user_id) {
                e.insert(PostUser {
                    user_id: r.user_id,
                    username: r.username,
                    profile_tag: r.profile_tag,
                    is_avatar_set: r.is_avatar_set,
                    is_admin: r.is_admin
                });
            }
        }
        posts.push(current);
        return Ok(serde_json::to_value(ThreadData {
            posts,
            users
        })?.into());
    }
    if sqlx::query!(
        "SELECT 1 AS ex FROM threads WHERE thread_id = ?",
        thread_id
    ).fetch_optional(&req.state().db).await?.is_some() {
        return Ok(serde_json::to_value(ThreadData { posts: vec![], users: HashMap::new() })?.into())
    }
    Ok(StatusCode::NotFound.into())
}
