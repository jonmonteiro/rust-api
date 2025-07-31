use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde_json::json;
use sqlx::MySqlPool;
use sqlx::Arguments; // Import this trait to use `add`

use crate::models::task::{CreateTaskReq, TaskRow, UpdateTaskReq};

pub async fn get_tasks(
    State(db_pool): State<MySqlPool>,
) -> Result<(StatusCode, String), (StatusCode, String)> {
    let rows = sqlx::query_as!(TaskRow, "SELECT * FROM tasks ORDER BY task_id")
        .fetch_all(&db_pool)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                json!({"success": false, "message": e.to_string()}).to_string(),
            )
        })?;

    Ok((
        StatusCode::OK, 
        json!({"success": true, "data": rows}).to_string(),
    ))
}

pub async fn get_task_by_id(
    State(db_pool): State<MySqlPool>,
    Path(task_id): Path<i32>,
) -> Result<(StatusCode, String), (StatusCode, String)> {
    let rows = sqlx::query_as!(TaskRow,"SELECT * FROM tasks WHERE task_id = ?", task_id,)
        .fetch_one(&db_pool)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                json!({"success": false, "message": e.to_string()}).to_string(),
            )
        })?;

    Ok((
        StatusCode::OK,
        json!({"success":true, "data": rows}).to_string()))
}

pub async fn create_task(
    State(db_pool): State<MySqlPool>,
    Json(task): Json<CreateTaskReq>,
) -> Result<(StatusCode, String), (StatusCode, String)> {
    let row = sqlx::query!(
        "INSERT INTO tasks (name, priority) VALUES (?, ?)",
        task.name,
        task.priority
    )
    .execute(&db_pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            json!({"success": false, "message": e.to_string()}).to_string(),
        )
    })?;

    let inserted_id = row.last_insert_id();

    Ok((
        StatusCode::CREATED,
        json!({"success": true, "data": { "task_id": inserted_id }}).to_string(),
    ))
}

pub async fn update_task(
    State(db_pool): State<MySqlPool>,
    Path(task_id): Path<i32>,
    Json(task): Json<UpdateTaskReq>,
) -> Result<(StatusCode, String), (StatusCode, String)> {
    let mut query = String::from("UPDATE tasks SET ");
    let mut updates = Vec::new();
    let mut args = sqlx::mysql::MySqlArguments::default();

    if let Some(name) = &task.name {
        updates.push("name = ?");
        args.add(name);
    }

    if let Some(priority) = task.priority {
        updates.push("priority = ?");
        args.add(priority);
    }

    if updates.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            json!({"success": false, "message": "Nothing to update"}).to_string(),
        ));
    }

    query.push_str(&updates.join(", "));
    query.push_str(" WHERE task_id = ?");
    args.add(task_id);

    sqlx::query_with(&query, args)
        .execute(&db_pool)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                json!({"success": false, "message": e.to_string()}).to_string(),
            )
        })?;

    Ok((StatusCode::OK, json!({"success": true}).to_string()))
}

pub async fn delete_task(
    State(db_pool): State<MySqlPool>,
    Path(task_id): Path<i32>,
) -> Result<(StatusCode, String), (StatusCode, String)> {
    sqlx::query!("DELETE FROM tasks WHERE task_id = ?", task_id,)
        .execute(&db_pool)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                json!({"success": false, "message": e.to_string()}).to_string(),
            )
        })?;

    Ok((StatusCode::OK, json!({"success":true}).to_string()))
}
