use chrono::offset::Utc;
use sqlx::{Error as SqlError, Postgres, Transaction};
use task::{Task, TaskState, TaskType};

#[rocket::async_trait]
pub trait ProcessTask {
    async fn get_open_task(executor: &mut Transaction<Postgres>) -> Result<Option<Task>, SqlError>;

    async fn set_running(&self, executor: &mut Transaction<Postgres>) -> Result<(), SqlError>;

    async fn set_finished(&self, executor: &mut Transaction<Postgres>) -> Result<(), SqlError>;
}

#[rocket::async_trait]
impl ProcessTask for Task {
    async fn get_open_task(executor: &mut Transaction<Postgres>) -> Result<Option<Task>, SqlError> {
        let timestamp = Utc::now().naive_utc();

        let query = sqlx::query_as!(
            Task,
            r#"
        SELECT
            id,
            task_type as "task_type: TaskType",
            task_state as "task_state: TaskState",
            sched_datetime
        FROM Tasks
        WHERE sched_datetime <= $1
          AND task_state = $2
        ORDER BY sched_datetime
        LIMIT 1
        FOR UPDATE
        SKIP LOCKED
        "#,
            timestamp,
            TaskState::Created.as_ref()
        );

        query.fetch_optional(executor).await
    }

    async fn set_running(&self, executor: &mut Transaction<Postgres>) -> Result<(), SqlError> {
        let query = sqlx::query!(
            r#"
            UPDATE Tasks
            SET task_state = $1
            WHERE id = $2
            "#,
            TaskState::Running.as_ref(),
            self.id
        );

        query.execute(executor).await?;
        Ok(())
    }

    async fn set_finished(&self, executor: &mut Transaction<Postgres>) -> Result<(), SqlError> {
        let query = sqlx::query!(
            r#"
            UPDATE Tasks
            SET task_state = $1
            WHERE id = $2
            "#,
            TaskState::Finished.as_ref(),
            self.id
        );

        query.execute(executor).await?;
        Ok(())
    }
}
