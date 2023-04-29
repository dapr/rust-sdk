use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct RegisteredActorsResponse {
    pub entities: Vec<String>,
}
