use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Schemas {
    pub input: serde_json::Value,
    pub output: serde_json::Value,
}
