use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct TaskRow {
    pub task_id: i32,
    pub name: String,
    pub priority: Option<i32>,
}

#[derive(Deserialize)]
pub struct CreateTaskReq {
    pub name: String,
    pub priority: Option<i32>,
}

#[derive(Deserialize)]
pub struct UpdateTaskReq {
    pub name: Option<String>,
    pub priority: Option<i32>,
}
