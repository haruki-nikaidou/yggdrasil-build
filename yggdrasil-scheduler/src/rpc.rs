use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use yggdrasil_bus::NatsJsonMessage;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddScheduleRequest {
    pub time: NaiveDateTime,
    pub future_subject: String,
    pub future_message: serde_json::Value,
}

#[async_trait::async_trait]
impl NatsJsonMessage for AddScheduleRequest {
    fn subject() -> &'static str {
        "yggdrasil-scheduler.add"
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddScheduleResponse {
    pub id: i64,
}

impl NatsJsonMessage for AddScheduleResponse {
    fn subject() -> &'static str {
        "yggdrasil-scheduler.add"
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteScheduleRequest {
    pub id: i64,
}

impl NatsJsonMessage for DeleteScheduleRequest {
    fn subject() -> &'static str {
        "yggdrasil-scheduler.delete"
    }
}
