use sqlx::{MySql, Pool};
use tide::{log, sessions::{Session, SessionStore}};


/// A workaround for tide issue #762.
/// https://github.com/http-rs/tide/issues/762
/// Credit: https://github.com/http-rs/tide/issues/762#issuecomment-808829054
pub trait SessionWorkaroundExt {
    /// Session key of regeneration flag.
    const REGENERATION_MARK_KEY: &'static str;

    /// Marks the session for ID regeneration.
    fn mark_for_regenerate(&mut self);

    /// Checks whether the session should regenerate the ID.
    /// The session key `REGENERATION_MARK` will be removved.
    fn should_regenerate(&mut self) -> bool;
}

impl SessionWorkaroundExt for Session {
    const REGENERATION_MARK_KEY: &'static str = "sid_regenerate";

    fn mark_for_regenerate(&mut self) {
        self.insert(Self::REGENERATION_MARK_KEY, true)
            .expect("Boolean should be serialized");
    }

    fn should_regenerate(&mut self) -> bool {
        let previously_changed = self.data_changed();
        let regenerate = self.get(Self::REGENERATION_MARK_KEY).unwrap_or_default();

        self.remove(Self::REGENERATION_MARK_KEY);
        if !previously_changed {
            self.reset_data_changed();
        }

        regenerate
    }
}

#[derive(Debug, Clone)]
pub struct MysqlSessionStore {
    pool: Pool<MySql>
}

impl MysqlSessionStore {
    pub fn new(pool: Pool<MySql>) -> MysqlSessionStore {
        MysqlSessionStore {
            pool
        }
    }
}

#[tide::utils::async_trait]
impl SessionStore for MysqlSessionStore {
    async fn load_session(&self, cookie_value: String) -> async_session::Result<Option<Session>> {
        let id = Session::id_from_cookie_value(&cookie_value)?;
        let rec = sqlx::query!(
            "SELECT data FROM sessions WHERE sess_id = ? AND (expiry IS NULL OR expiry > NOW())",
            id
        ).fetch_optional(&self.pool).await?;

        match rec {
            Some(r) => {
                let sess: Option<Session> = serde_json::from_str(&r.data)?;
                match &sess {
                    Some(s) => log::debug!("loaded session: {:?} | data_changed: {:?}", s, s.data_changed()),
                    None => log::warn!("failed to load session with id {}", id)
                }
                Ok(sess)
            },
            None => {
                log::info!("No session found by id {}", id);
                Ok(None)
            }
        }
    }

    async fn store_session(&self, mut session: Session) -> async_session::Result<Option<String>> {
        if session.should_regenerate() {
            session.regenerate();
        }
        return match sqlx::query!(
            "INSERT INTO sessions(sess_id, expiry, data) VALUES (?, ?, ?) AS new
             ON DUPLICATE KEY UPDATE expiry = new.expiry, data = new.data",
            session.id(), session.expiry(), serde_json::to_string(&session)?
        ).execute(&self.pool).await {
            Ok(_) => Ok(session.into_cookie_value()),
            Err(e) => Err(e.into())
        }
    }

    async fn destroy_session(&self, session: Session) -> async_session::Result {
        return match sqlx::query!("DELETE FROM sessions WHERE sess_id = ?", session.id())
            .execute(&self.pool).await {
            Ok(_) => Ok(()),
            Err(e) => Err(e.into())
        }
    }

    async fn clear_store(&self) -> async_session::Result {
        return match sqlx::query!("TRUNCATE sessions").execute(&self.pool).await {
            Ok(_) => Ok(()),
            Err(e) => Err(e.into())
        }
    }
}
