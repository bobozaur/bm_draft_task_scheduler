use sqlx::{Postgres, Transaction};
use task::TaskState;
use tokio::task::JoinHandle;

use crate::error::Error;

pub struct OpenTask<'a> {
    pub(crate) transaction: Transaction<'a, Postgres>,
}

impl<'a> OpenTask<'a> {
    pub fn new(transaction: Transaction<'a, Postgres>) -> Self {
        Self { transaction }
    }
}

pub struct RunningTask<'a, T> {
    pub(crate) transaction: Transaction<'a, Postgres>,
    pub(crate) handle: JoinHandle<Result<T, Error>>,
}

impl<'a, T> RunningTask<'a, T> {
    pub fn new(
        handle: JoinHandle<Result<T, Error>>,
        transaction: Transaction<'a, Postgres>,
    ) -> Self {
        Self {
            handle,
            transaction,
        }
    }
}

pub struct AbortedTask;

pub struct FinishedTask<T>(pub(crate) Result<T, Error>);

pub trait WorkerTaskState {
    fn validate_state(_: TaskState) -> Result<(), Error> {
        Err(Error::TaskStateError)
    }
}

impl<'a> WorkerTaskState for OpenTask<'a> {
    fn validate_state(task_state: TaskState) -> Result<(), Error> {
        match task_state {
            TaskState::Open => Ok(()),
            _ => Err(Error::TaskStateError),
        }
    }
}
impl<'a, T> WorkerTaskState for RunningTask<'a, T> {}
impl WorkerTaskState for AbortedTask {}
impl<T> WorkerTaskState for FinishedTask<T> {}
