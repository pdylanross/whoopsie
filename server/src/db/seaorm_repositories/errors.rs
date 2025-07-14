use app::types::RepositoryError;
use sea_orm::DbErr;

pub trait RepoErrExts<T> {
    fn to_repo_err(self) -> Result<T, RepositoryError>;
}

impl<T> RepoErrExts<T> for Result<T, DbErr> {
    fn to_repo_err(self) -> Result<T, RepositoryError> {
        match self {
            Ok(me) => Ok(me),
            Err(err) => Err(RepositoryError::InternalServerError(err.to_string())),
        }
    }
}
