use axum::{
    routing::{get},
    Router,
};
use sqlx::MySqlPool;

use crate::handlers::tasks::{get_tasks, create_task, update_task, delete_task, get_task_by_id};

pub fn tasks_routes(db_pool: MySqlPool) -> Router {
    Router::new()
        .route("/tasks", get(get_tasks).post(create_task))
        .route("/tasks/:task_id", 
            get(get_task_by_id)
                .put(update_task)
                .delete(delete_task)
        )
        .with_state(db_pool)
}
