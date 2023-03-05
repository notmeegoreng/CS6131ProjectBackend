pub mod models;
mod routes;

use async_std;
use sqlx::mysql::MySqlPoolOptions;

#[derive(Clone)]
pub struct State {
    db: sqlx::mysql::MySqlPool
}

pub type Request = tide::Request<State>;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    async_std::task::block_on(a_main())?;
    Ok(())
}

async fn a_main() -> Result<(), Box<dyn std::error::Error>> {
    let pool = MySqlPoolOptions::new()
        .max_connections(5)
        .connect("mysql://root:admin@localhost/forum_site").await?;
    let mut app = tide::with_state(State { db: pool.clone() });
    let mut users = app.at("/users");
    users.post(routes::users::create);
    users
        .at("/:user_id")
        .get(routes::users::get);

    app.listen("127.0.0.1:8000").await?;
    pool.close().await;
    Ok(())
}
