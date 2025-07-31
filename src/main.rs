use std::time::Duration;

use axum::{
  extract::{Path, State},
  http::StatusCode,
  routing::{get, put},
  Json, Router,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::{mysql::MySqlPoolOptions, MySqlPool};
use tokio::net::TcpListener;
use sqlx::Arguments;

#[tokio::main]
async fn main() {
  //expose environment variables from .env file
  dotenvy::dotenv().expect("Unable to access .env file");

  //set variables from enviroment variables
  let server_address = std::env::var("SERVER_ADDRESS").unwrap_or("127.0.0.1:3000".to_owned());
  let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL not found in env file");

  //create our database pool
  let db_pool = MySqlPoolOptions::new()
    .max_connections(64)
    .acquire_timeout(Duration::from_secs(5))
    .connect(&database_url)
    .await
    .expect("can't connect to database");

  //create our tcp listener
  let listener = TcpListener::bind(server_address)
    .await
    .expect("Could not create tcp listener");

  println!("listening on {}", listener.local_addr().unwrap());

  // compose the routes
  let app = Router::new()
    .route("/", get(|| async { "Hello world" }))
    .route("/tasks", get(get_tasks).post(create_task))
    .route("/tasks/:task_id", put(update_task).delete(delete_task).get(get_task_by_id))
    .with_state(db_pool);

  //serve the application
  axum::serve(listener, app)
    .await
    .expect("Error serving application");
}

async fn get_tasks(
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

async fn get_task_by_id(
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

async fn create_task(
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

async fn update_task(
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


async fn delete_task(
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


#[derive(Serialize)]
struct TaskRow {
  task_id: i32,
  name: String,
  priority: Option<i32>,
}

#[derive(Deserialize)]
struct CreateTaskReq {
  name: String,
  priority: Option<i32>,
}

#[derive(Deserialize)]
struct UpdateTaskReq {
  name: Option<String>,
  priority: Option<i32>,
}