use serde::Deserialize;

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
    ($n:ident, $rn:ident, $m:path, $q:literal, $($p:expr),*) => {
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

#[derive(Deserialize)]
pub struct SearchQuery {
    pub q: Option<String>
}

macro_rules! route_search {
    ($n:ident, $query:literal) => {
        $crate::utils::wrapper!($n, req, {
            let query = req.query::<crate::utils::SearchQuery>()?;

            if let Some(q) = query.q {
                let mut s = sqlx::query!($query, q).fetch(&req.state().db);
                return Ok(serde_json::to_value(data_into_hashmap!(s))?.into());
            }
            Ok(Response::builder(StatusCode::BadRequest)
                .content_type("text/plain")
                .body("Please provide a query `q`")
                .build())
        });
    };
}

macro_rules! data_into_hashmap {
    ($s:ident) => {
        {
            let mut data: HashMap::<u32, ContainerData> = HashMap::new();
            while let Some(Ok(r)) = $s.next().await {
                match data.entry(r.p_id) {
                    Entry::Occupied(mut e) => {
                        e.get_mut().children.push(Container {
                            id: r.c_id, name: r.c_name, description: r.c_descr
                        });
                    },
                    Entry::Vacant(e) => {
                        e.insert(ContainerData {
                            container: BasicContainer { name: r.p_name, description: r.p_descr },
                            children: vec!(Container {
                                id: r.c_id, name: r.c_name, description: r.c_descr
                            })
                        });
                    }
                }
            }
            data
        }
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
pub(crate) use route_search;
pub(crate) use data_into_hashmap;
pub(crate) use wrap_error;
