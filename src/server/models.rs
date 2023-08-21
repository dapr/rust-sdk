use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct RegisteredActorsResponse {
    pub entities: Vec<String>,
}
