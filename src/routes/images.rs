

use serde::{Serialize, Deserialize};

#[derive(Debug, Eq, PartialEq, Clone, Serialize, Deserialize)]
struct Image {
    name: String,
    data: Vec<u8>
}

