use serde::Serialize;

#[derive(Debug, Eq, PartialEq, Clone, Serialize)]
pub struct User {
    pub user_id: u32,
    pub username: String,
    pub description: String,
    pub profile_tag: String,
    // password hash is not stored on the main struct as it should only be used on login
    pub is_avatar_set: bool,
    pub is_admin: bool
}

#[derive(Debug, Eq, PartialEq, Clone, Serialize)]
pub struct Log {
    pub log_id: u32,
    pub log: String,
    pub time: chrono::NaiveDateTime
}
