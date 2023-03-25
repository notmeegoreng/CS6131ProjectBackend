pub(crate) mod auth;
pub(crate) mod macros;
pub(crate) mod sessions;

use std::fmt::{Debug, Display, Formatter};
pub(crate) use macros::wrapper;
// pub(crate) use macros::wrapper_mut;
pub(crate) use macros::route_get;

// pub(crate) use macros::route_post;
pub(crate) use macros::wrap_error;


pub struct Error {
    pub(crate) msg: &'static str
}

impl Debug for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Error: {}", self.msg)
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Error: {}", self.msg)
    }
}

impl std::error::Error for Error {}

impl Error {
    /*
    pub(crate) fn new(msg: &'static str) -> Self {
        Self {
            msg
        }
    }
    */
}
