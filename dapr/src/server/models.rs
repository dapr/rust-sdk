use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq)]
pub struct ReentrancyConfig {
    pub enabled: bool,
    #[serde(skip_serializing_if = "Option::is_none", rename = "maxStackDepth")]
    pub max_stack_depth: Option<i32>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq)]
pub struct EntityConfig {
    pub entities: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reentrancy: Option<ReentrancyConfig>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq)]
pub struct RegisteredActorsResponse {
    pub entities: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reentrancy: Option<ReentrancyConfig>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "entitiesConfig")]
    pub entities_config: Option<Vec<EntityConfig>>,
}
