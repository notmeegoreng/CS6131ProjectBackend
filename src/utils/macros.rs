macro_rules! wrapper {
    ($n:ident, $rn:ident, $b:block) => {
        pub async fn $n($rn: $crate::Request) -> tide::Result $b
    }
}
/*
macro_rules! wrapper_mut {
    ($n:ident, $rn:ident, $b:block) => {
        pub async fn $n(mut $rn: $crate::Request) -> tide::Result $b
    }
}
*/
macro_rules! route_get {
    ($n:ident, $rn:ident, $m:path, $q:expr, $($p:expr),*) => {
        $crate::utils::wrapper!($n, $rn, {
            let rec = sqlx::query_as!($m, $q, $($p),*)
                .fetch_optional(&$rn.state().db).await?;
            if let Some(d) = rec {
                return Ok(serde_json::to_value(d)?.into())
            }
            return Ok(StatusCode::NotFound.into())
        });
    };
    ($n:ident, $rn:ident, $b:block) => {
        $crate::utils::wrapper!($n, $rn, {
            let d = $b;
            Ok(serde_json::to_value(d)?.into())
        });
    };
}

macro_rules! wrap_error {
    ($e:expr, $s:path) => {
        match $e {
            Ok(o) => o,
            Err(e) => return Err(tide::Error::new($s, e))
        }
    };
    ($e:expr, $n:ident, $b:block, $s:ident) => {
        match $e {
            Ok($n) => $b,
            Err(e) => return Err(tide::Error::new($s, e))
        }
    };
}

pub(crate) use wrapper;
//pub(crate) use wrapper_mut;
pub(crate) use route_get;
//pub(crate) use route_post;
pub(crate) use wrap_error;
