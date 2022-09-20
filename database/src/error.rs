use rocket::figment::Error as FigmentError;
use sqlx::Error as SqlError;
use thiserror::Error as ThisError;

#[allow(variant_size_differences)]
#[derive(Debug, ThisError)]
pub enum InitPoolError {
    #[error(transparent)]
    Config(#[from] FigmentError),
    #[error(transparent)]
    Sqlx(#[from] SqlError),
}
