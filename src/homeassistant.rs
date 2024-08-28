use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct EntityInfo {
    pub component: String,
    pub name: String,
    pub payload_off: String,
    pub payload_on: String,
    pub state_topic: String,
    pub json_attributes_topic: String,
    pub command_topic: String,
    pub unique_id: String,
}
