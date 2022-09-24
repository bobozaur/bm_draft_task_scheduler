use std::fmt::{Display, Formatter, Result as FmtResult};

use chrono::NaiveDateTime;
use rocket::{
    serde::{uuid::Uuid, Deserialize, Serialize},
    FromFormField,
};
use sqlx::{Error as SqlError, PgPool, Type};
use strum_macros::AsRefStr;

#[derive(AsRefStr, Clone, Copy, Deserialize, Debug, FromFormField, Serialize, Type)]
#[serde(crate = "rocket::serde")]
pub enum TaskType {
    A,
    B,
    C,
}

impl Display for TaskType {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}", self.as_ref())
    }
}

#[derive(AsRefStr, Clone, Copy, Deserialize, Debug, FromFormField, Serialize, Type)]
#[serde(crate = "rocket::serde")]
pub enum TaskState {
    Open,
    Running,
    Aborted,
    Failed,
    Successful,
}

impl Display for TaskState {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}", self.as_ref())
    }
}

impl Default for TaskState {
    fn default() -> Self {
        Self::Open
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct Task {
    #[serde(default = "Uuid::new_v4")]
    pub id: Uuid,
    pub task_type: TaskType,
    #[serde(default)]
    pub task_state: TaskState,
    #[serde(with = "datetime_deser")]
    pub sched_datetime: NaiveDateTime,
}

impl Task {
    pub async fn get(id: Uuid, pool: &PgPool) -> Result<Option<Self>, SqlError> {
        let query = sqlx::query_as!(
            Self,
            r#"
            SELECT
                id,
                task_type as "task_type: TaskType",
                task_state as "task_state: TaskState",
                sched_datetime
            FROM Tasks
            WHERE id = $1
            "#,
            id
        );

        query.fetch_optional(pool).await
    }

    pub async fn get_filtered(
        task_type: Option<TaskType>,
        task_state: Option<TaskState>,
        pool: &PgPool,
    ) -> Result<Vec<Self>, SqlError> {
        let query = sqlx::query_as!(
            Self,
            r#"
            SELECT
                id,
                task_type as "task_type: TaskType",
                task_state as "task_state: TaskState",
                sched_datetime
            FROM Tasks
            WHERE task_type = CASE WHEN $1::text IS NULL THEN task_type ELSE $1 END
              OR task_state = CASE WHEN $2::text IS NULL THEN task_type ELSE $2 END
            "#,
            task_type.as_ref().map(|v| v.as_ref()),
            task_state.as_ref().map(|v| v.as_ref())
        );

        query.fetch_all(pool).await
    }

    // TODO: Some SQL error parsing is necessary to
    // properly handle the scenario where the UUID already exists.
    //
    // This is a bit more complicated because the constraint name has to be used.
    pub async fn insert(&self, pool: &PgPool) -> Result<(), SqlError> {
        let query = sqlx::query!(
            r#"
            INSERT INTO Tasks VALUES ($1, $2, $3, $4)
            "#,
            &self.id,
            self.task_type.as_ref(),
            self.task_state.as_ref(),
            &self.sched_datetime
        );

        query.execute(pool).await?;
        Ok(())
    }
    // TODO: Some SQL error parsing is necessary to
    // properly handle the scenario where the UUID does not exist.
    //
    // The number of affected rows can be used to determine whether
    // the ID already existed or not.
    pub async fn delete(id: Uuid, pool: &PgPool) -> Result<(), SqlError> {
        let query = sqlx::query!(
            r#"
            DELETE FROM Tasks
            WHERE id = $1
            "#,
            id
        );

        query.execute(pool).await?;
        Ok(())
    }
}

mod datetime_deser {
    use chrono::NaiveDateTime;
    use rocket::serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(date: &NaiveDateTime, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_i64(date.timestamp())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<NaiveDateTime, D::Error>
    where
        D: Deserializer<'de>,
    {
        let secs = i64::deserialize(deserializer)?;
        Ok(NaiveDateTime::from_timestamp(secs, 0))
    }
}
