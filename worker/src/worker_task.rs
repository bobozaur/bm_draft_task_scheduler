use std::time::Duration;

use chrono::{offset::Utc, NaiveDateTime};
use rocket::tokio::time;
use sqlx::{Error as SqlError, PgPool, Postgres, Transaction};
use task::{Task, TaskState, TaskType};
use uuid::Uuid;

use crate::{error::Error, task_state::{WorkerTaskState, OpenTask, RunningTask, AbortedTask, FinishedTask}};

pub struct WorkerTask<T>
where
    T: WorkerTaskState,
{
    id: Uuid,
    task_type: TaskType,
    task_state: T,
    sched_datetime: NaiveDateTime,
}

impl<'a> WorkerTask<OpenTask<'a>> {
    fn from_task(task: Task, transaction: Transaction<'a, Postgres>) -> Result<Self, Error> {
        OpenTask::validate_state(task.task_state)?;

        let worker_task = Self {
            id: task.id,
            task_type: task.task_type,
            task_state: OpenTask::new(transaction),
            sched_datetime: task.sched_datetime,
        };

        Ok(worker_task)
    }
    async fn get_task(executor: &mut Transaction<'_, Postgres>) -> Result<Option<Task>, SqlError> {
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
            TaskState::Open.as_ref()
        );

        query.fetch_optional(executor).await
    }

    pub async fn get_open_task(pool: &PgPool) -> Result<Option<WorkerTask<OpenTask<'a>>>, Error> {
        let mut transaction = pool.begin().await?;

        let opt_task = Self::get_task(&mut transaction).await?;

        let task = match opt_task {
            Some(task) => Some(Self::from_task(task, transaction)?),
            None => None,
        };

        Ok(task)
    }

    async fn set_running(&mut self) -> Result<(), SqlError> {
        let query = sqlx::query!(
            r#"
            UPDATE Tasks
            SET task_state = $1
            WHERE id = $2
            "#,
            TaskState::Running.as_ref(),
            self.id
        );

        query.execute(&mut self.task_state.transaction).await?;
        Ok(())
    }

    /// This wouldn't be so easy to do in a real scenario.
    /// We're basically taking advantage of the fact that types are Copy here.
    async fn process_task(id: Uuid, task_type: TaskType) -> Result<(), Error> {
        time::sleep(Duration::from_secs(5)).await;
        println!("Ran task ID: {} - type: {}", id, task_type);
        Ok(())
    }

    pub async fn start<'b>(mut self) -> Result<WorkerTask<RunningTask<'b, ()>>, SqlError>
    where
        'a: 'b,
    {
        self.set_running().await?;
        let handle = tokio::spawn(Self::process_task(self.id, self.task_type));

        let running_task = WorkerTask {
            id: self.id,
            task_type: self.task_type,
            task_state: RunningTask::new(handle, self.task_state.transaction),
            sched_datetime: self.sched_datetime,
        };

        Ok(running_task)
    }
}

impl<'a, T> WorkerTask<RunningTask<'a, T>> {
    pub async fn abort(self) -> Result<WorkerTask<AbortedTask>, Error> {
        let RunningTask {
            mut transaction,
            handle,
        } = self.task_state;

        handle.abort();

        Self::set_aborted(self.id, &mut transaction).await?;

        transaction.commit().await?;

        let task = WorkerTask {
            id: self.id,
            task_type: self.task_type,
            task_state: AbortedTask,
            sched_datetime: self.sched_datetime,
        };

        Ok(task)
    }

    pub fn is_finished(&self) -> bool {
        self.task_state.handle.is_finished()
    }

    async fn set_state(
        task_state: TaskState,
        id: Uuid,
        transaction: &mut Transaction<'_, Postgres>,
    ) -> Result<(), SqlError> {
        let query = sqlx::query!(
            r#"
            UPDATE Tasks
            SET task_state = $1
            WHERE id = $2
            "#,
            task_state.as_ref(),
            id
        );

        query.execute(transaction).await?;
        Ok(())
    }

    async fn set_aborted(
        id: Uuid,
        transaction: &mut Transaction<'_, Postgres>,
    ) -> Result<(), SqlError> {
        Self::set_state(TaskState::Aborted, id, transaction).await
    }

    async fn set_failed(
        id: Uuid,
        transaction: &mut Transaction<'_, Postgres>,
    ) -> Result<(), SqlError> {
        Self::set_state(TaskState::Failed, id, transaction).await
    }

    async fn set_success(
        id: Uuid,
        transaction: &mut Transaction<'_, Postgres>,
    ) -> Result<(), SqlError> {
        Self::set_state(TaskState::Successful, id, transaction).await
    }

    pub async fn finish(self) -> Result<WorkerTask<FinishedTask<T>>, Error> {
        // The error here is from joining on the task handle.
        let RunningTask {
            mut transaction,
            handle,
        } = self.task_state;

        let result = handle.await?;

        match &result {
            Ok(_) => Self::set_success(self.id, &mut transaction).await?,
            Err(_) => Self::set_failed(self.id, &mut transaction).await?,
        };

        transaction.commit().await?;

        let task = WorkerTask {
            id: self.id,
            task_type: self.task_type,
            task_state: FinishedTask(result),
            sched_datetime: self.sched_datetime,
        };

        Ok(task)
    }
}

impl<T> WorkerTask<FinishedTask<T>> {
    pub fn into_result(self) -> Result<T, Error> {
        self.task_state.0
    }
}
