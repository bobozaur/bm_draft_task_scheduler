use sqlx::Error as SqlError;
use thiserror::Error as ThisError;
use tokio::task::JoinError;

#[derive(Debug, ThisError)]
pub enum Error {
    #[error(transparent)]
    JoinError(#[from] JoinError),
    #[error(transparent)]
    SqlError(#[from] SqlError),
    #[error("Invalid task state")]
    TaskStateError,
}
