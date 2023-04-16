use tide::{Response, Server, StatusCode};
use tide::sessions;
use tide::http::cookies::SameSite;

use crate::{Request, State, utils};

mod users;
mod containers;
mod containers_modify;
mod images;
mod auth;
mod search;
mod reactions;

async fn ok(_: Request) -> tide::Result {
    Ok(Response::new(StatusCode::NoContent))
}

pub fn add_routes(app: &mut Server<State>, image_file_dir: &str) {
    let db = app.state().db.clone();
    app.at("/images/").serve_dir(image_file_dir).expect("invalid directory");
    let mut api = app.at("/api");
    api.get(ok);
    api.with(sessions::SessionMiddleware::new(
        utils::sessions::MysqlSessionStore::new(db),
        hex::decode(
            std::env::var("SESSION_SECRET")
                .expect("SESSION_SECRET env var should be set")
        ).expect("SESSION_SECRET should contain valid hex").as_slice()
    ).with_same_site_policy(SameSite::Lax).with_cookie_name("10_c"));

    let mut images = api.at("/images");
    images.at("/set_avatar").post(images::set_avatar);

    api.at("/home").get(containers::home);
    api.at("/latest").get(containers::latest_posts);

    let mut categories = api.at("/categories");
    categories.get(containers::all_categories).post(containers_modify::category_create);
    categories.at("/:category_id")
        .patch(containers_modify::category_patch)
        .delete(containers_modify::category_delete);

    let mut forums = api.at("/forums");
    forums.get(containers::all_forums).post(containers_modify::forum_create);
    forums.at("/:forum_id")
        .get(containers::forum_data)
        .patch(containers_modify::forum_patch)
        .delete(containers_modify::forum_delete);

    let mut topics = api.at("/topics");
    topics.get(containers::all_topics).post(containers_modify::topic_create);
    topics
        .at("/:topic_id")
            .get(containers::topic_info)
            .patch(containers_modify::topic_patch)
            .delete(containers_modify::topic_delete)
        .at("/page/:page_num").get(containers::topic_pages);

    let mut threads = api.at("/threads");
    threads.post(containers_modify::thread_create);
    threads
        .at("/:thread_id")
            .get(containers::thread_info)
            .delete(containers_modify::thread_delete)
        .at("/page/:page_num").get(containers::thread_pages);

    let mut posts = api.at("/posts");
    posts.post(containers_modify::post_create);
    let mut post_specific = posts.at("/:post_id");
    post_specific.delete(containers_modify::post_delete);
    post_specific.at("/reactions/add").post(reactions::add_reaction);
    post_specific.at("/reactions/rem").post(reactions::rem_reaction);

    let mut users = api.at("/users");
    users.at("/available").get(users::available_username);
    let mut user_specific = users.at("/:user_id");
    user_specific
        .get(users::user_get)
        .patch(users::user_patch);
    user_specific.at("/logs").get(users::log_get);

    let mut auth = api.at("/auth");
    auth.at("/pre_auth").post(auth::pre_auth);
    auth.at("/register").post(auth::register);
    auth.at("/login").post(auth::login);
    auth.at("/logout").post(auth::logout);

    let mut search = api.at("/search");
    search.at("/users").get(search::user_search);
    search.at("/forums").get(search::forum_search);
    search.at("/topics").get(search::topic_search);
    search.at("/threads").get(search::thread_search);
    search.at("/posts").get(search::post_search);
}
