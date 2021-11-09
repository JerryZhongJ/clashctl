use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "lowercase")]
pub enum Level {
    Error,
    Warning,
    Info,
    Debug,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Log {
    #[serde(rename = "type")]
    pub log_type: Level,
    pub payload: String,
}
