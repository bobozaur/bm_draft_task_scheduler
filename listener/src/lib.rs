use rocket::{
    delete, get,
    http::Status,
    post,
    serde::{json::Json, uuid::Uuid},
    State,
};
use sqlx::PgPool;
use task::{Task, TaskState, TaskType};

#[get("/task/<id>")]
pub async fn retrieve_task(id: Uuid, pool: &State<PgPool>) -> Result<Option<Json<Task>>, Status> {
    Task::get(id, pool)
        .await
        .map(|opt| opt.map(Json))
        .map_err(|_| Status::InternalServerError)
}

#[get("/task?<task_type>&<task_state>")]
pub async fn retrieve_filtered_tasks(
    task_type: Option<TaskType>,
    task_state: Option<TaskState>,
    pool: &State<PgPool>,
) -> Result<Json<Vec<Task>>, Status> {
    Task::get_filtered(task_type, task_state, pool)
        .await
        .map(Json)
        .map_err(|_| Status::InternalServerError)
}

#[post("/task", format = "json", data = "<task>")]
pub async fn create_task(task: Json<Task>, pool: &State<PgPool>) -> Result<Json<Uuid>, Status> {
    task.insert(pool)
        .await
        .map_err(|_| Status::InternalServerError)?;

    Ok(Json(task.id))
}

#[delete("/task/<id>")]
pub async fn delete_task(id: Uuid, pool: &State<PgPool>) -> Result<(), Status> {
    Task::delete(id, pool)
        .await
        .map_err(|_| Status::InternalServerError)
}
