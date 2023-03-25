use tide::Server;
use tide::sessions;
use tide::http::cookies::SameSite;

use crate::{State, utils};

mod users;
mod containers;
mod containers_create;
mod images;
mod auth;
mod search;

pub fn add_routes(app: &mut Server<State>, image_file_dir: &str) {
    let db = app.state().db.clone();
    app.at("/images/:image_id").serve_dir(image_file_dir).expect("invalid directory");
    let mut api = app.at("/api");
    api.with(sessions::SessionMiddleware::new(
        utils::sessions::MysqlSessionStore::new(db),
        hex::decode(
            std::env::var("SESSION_SECRET")
                .expect("SESSION_SECRET env var should be set")
        ).expect("SESSION_SECRET should contain valid hex").as_slice()
    ).with_same_site_policy(SameSite::Lax).with_cookie_name("10_c"));

    api.at("/home").get(containers::home);

    let mut categories = api.at("/categories");
    categories.post(containers_create::category_create);

    let mut forums = api.at("/forums");
    forums.post(containers_create::forum_create);
    forums.at("/:forum_id").get(containers::forum_data);

    let mut topics = api.at("/topics");
    topics.post(containers_create::topic_create);
    topics
        .at("/:topic_id").get(containers::topic_info)
        .at("/page/:page_num").get(containers::topic_pages);

    let mut threads = api.at("/threads");
    threads.post(containers_create::thread_create);
    threads
        .at("/:thread_id").get(containers::thread_info)
        .at("/page/:page_num").get(containers::thread_pages);

    api.at("/posts").post(containers_create::post_create);

    let mut users = api.at("/users");
    users.at("/available").post(users::available_username);
    let mut user_specific = users.at("/:user_id");
    user_specific.get(users::user_get);
    user_specific.at("/logs").get(users::log_get);

    let mut auth = api.at("/auth");
    auth.at("/pre_auth").post(auth::pre_auth);
    auth.at("/register").post(auth::register);
    auth.at("/login").post(auth::login);
    auth.at("/logout").post(auth::logout);

    let mut search = api.at("/search");
    search.at("/users").get(search::user_search);
    search.at("/categories").get(search::category_search);
    search.at("/forums").get(search::forum_search);
    search.at("/topics").get(search::topic_search);
}
