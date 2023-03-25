mod routes;
mod utils;
mod middleware;
mod models;

use async_std;
use sqlx::mysql::MySqlPoolOptions;
use tide::http::headers::HeaderValue;
use tide::log;

#[derive(Clone)]
pub struct State {
    db: sqlx::mysql::MySqlPool
}

pub type Request = tide::Request<State>;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // startup
    dotenvy::dotenv().expect(".env file not found");
    log::with_level(log::LevelFilter::Debug); // log everything

    // app construction and block
    async_std::task::block_on(async {
        let pool = MySqlPoolOptions::new()
            .max_connections(5)
            .connect("mysql://root:admin@localhost/forum_site").await?;
        let mut app = tide::with_state(State { db: pool.clone() });
        routes::add_routes(
            &mut app
                .with(middleware::ErrorHandleMiddleware {})
                .with(log::LogMiddleware::new())
                .with(tide::security::CorsMiddleware::new()
                    .allow_credentials(true)
                    .allow_headers("Content-Type".parse::<HeaderValue>()?)
                    .allow_origin("http://localhost:1212")
                    .expose_headers("Content-Encoding".parse::<HeaderValue>()?)),
            r#".\images\"#
        );

        app.listen("127.0.0.1:1414").await?;
        pool.close().await;
        Ok(())
    })
}
