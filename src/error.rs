use axum::{Json, http::StatusCode, response::IntoResponse};
use deadpool_diesel::{InteractError, postgres::PoolError};
use diesel::{result::DatabaseErrorKind, result::Error as DieselErr};
use jsonwebtoken::errors::ErrorKind as JwtErr;

use exn::Exn;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Error {
    #[serde(rename = "type")]
    error_type: Source,
    message: String,
}

#[derive(Clone, Copy, Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Source {
    Authentication(AuthenticationError),
    Database(DatabaseError),
    Internal,
}

#[derive(Clone, Copy, Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuthenticationError {
    InvalidFormat,
    InvalidIssuer,
    InvalidAlgorythm,
    InvalidSignature,
    InvalidKey,
    InvalidClaim,
    Missing,
    NoMatchingKey,
    InvalidTimeRange,
    InsufficientPermissions,
    NotInNetwork,
    Unknown,
}

#[derive(Clone, Copy, Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DatabaseError {
    Timeout,
    Connection,
    Pool,
    NotFound,
    Transaction,
    Serialization,
    Query,
    ConstraintViolation,
    Unknown,
}

impl Error {
    #[must_use]
    pub const fn authentication(reason: AuthenticationError, message: String) -> Self {
        Self {
            error_type: Source::Authentication(reason),
            message,
        }
    }

    #[must_use]
    pub const fn database(reason: DatabaseError, message: String) -> Self {
        Self {
            error_type: Source::Database(reason),
            message,
        }
    }

    #[must_use]
    pub const fn upstream(message: String) -> Self {
        Self {
            error_type: Source::Internal,
            message,
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}: {}", self.error_type, self.message)
    }
}

impl std::error::Error for Error {}

impl From<jsonwebtoken::errors::Error> for Error {
    fn from(value: jsonwebtoken::errors::Error) -> Self {
        match value.into_kind() {
            JwtErr::InvalidToken
            | JwtErr::InvalidKeyFormat
            | JwtErr::Base64(_)
            | JwtErr::Json(_)
            | JwtErr::Utf8(_) => Self {
                error_type: Source::Authentication(AuthenticationError::InvalidFormat),
                message: "JWT could not be decoded or deserialized".into(),
            },

            JwtErr::InvalidSignature => Self {
                error_type: Source::Authentication(AuthenticationError::InvalidSignature),
                message: "JWT signature does not match".into(),
            },

            JwtErr::InvalidEcdsaKey | JwtErr::InvalidEddsaKey | JwtErr::InvalidRsaKey(_) => Self {
                error_type: Source::Authentication(AuthenticationError::InvalidSignature),
                message: "Cryptographic key is not correctly formatted".into(),
            },

            JwtErr::InvalidAlgorithmName | JwtErr::InvalidAlgorithm | JwtErr::MissingAlgorithm => {
                Self {
                    error_type: Source::Authentication(AuthenticationError::InvalidAlgorythm),
                    message: "Algorythm does not match key id".into(),
                }
            }

            JwtErr::MissingRequiredClaim(err) | JwtErr::InvalidClaimFormat(err) => Self {
                error_type: Source::Authentication(AuthenticationError::InvalidClaim),
                message: format!("Missing or invalid claim(s): {err}"),
            },

            JwtErr::ExpiredSignature | JwtErr::ImmatureSignature => Self {
                error_type: Source::Authentication(AuthenticationError::InvalidTimeRange),
                message: "JWT is expired or immature. Re-authentication is required".into(),
            },

            JwtErr::InvalidIssuer => Self {
                error_type: Source::Authentication(AuthenticationError::InvalidIssuer),
                message: "JWT is not issued by a trusted source".into(),
            },

            JwtErr::InvalidAudience => Self {
                error_type: Source::Authentication(AuthenticationError::InsufficientPermissions),
                message: "This JWT does not grant permissions for this operation".into(),
            },

            _ => Self {
                error_type: Source::Authentication(AuthenticationError::Unknown),
                message: "Something unexpected happened during authentication".into(),
            },
        }
    }
}

impl From<PoolError> for Error {
    fn from(value: PoolError) -> Self {
        match value {
            PoolError::Timeout(_) => Self {
                error_type: Source::Database(DatabaseError::Timeout),
                message: "Timed out while trying to communicate to database".into(),
            },

            PoolError::Backend(_) => Self {
                error_type: Source::Database(DatabaseError::Connection),
                message: "Connection to the database failed".into(),
            },

            _ => Self {
                error_type: Source::Database(DatabaseError::Pool),
                message: "Database pool closed or falied to open".into(),
            },
        }
    }
}

impl From<InteractError> for Error {
    fn from(_value: InteractError) -> Self {
        Self {
            error_type: Source::Database(DatabaseError::Pool),
            message: "Interaction with database pool failed".into(),
        }
    }
}

impl From<DieselErr> for Error {
    fn from(value: diesel::result::Error) -> Self {
        match value {
            DieselErr::NotFound => Self {
                error_type: Source::Database(DatabaseError::NotFound),
                message: "What was requested wasn't found on the database".into(),
            },

            DieselErr::RollbackErrorOnCommit { .. }
            | DieselErr::RollbackTransaction
            | DieselErr::AlreadyInTransaction
            | DieselErr::NotInTransaction
            | DieselErr::BrokenTransactionManager
            | DieselErr::DatabaseError(
                DatabaseErrorKind::UnableToSendCommand | DatabaseErrorKind::ReadOnlyTransaction,
                _,
            ) => Self {
                error_type: Source::Database(DatabaseError::Transaction),
                message: format!(
                    "Something went wrong while dealing with a transaction: {value:?}"
                ),
            },

            DieselErr::InvalidCString(_)
            | DieselErr::DatabaseError(DatabaseErrorKind::SerializationFailure, _) => Self {
                error_type: Source::Database(DatabaseError::Serialization),
                message: format!(
                    "Something went wrong trying to convert to or from a format the database can understand: {value:?}"
                ),
            },

            DieselErr::SerializationError(err) | DieselErr::DeserializationError(err) => Self {
                error_type: Source::Database(DatabaseError::Serialization),
                message: format!(
                    "Something went wrong trying to convert to or from a format the database can understand: {err:?}"
                ),
            },

            DieselErr::QueryBuilderError(_) => Self {
                error_type: Source::Database(DatabaseError::Query),
                message: format!("A query was submitted that's not possible to execute: {value:?}"),
            },

            DieselErr::DatabaseError(
                DatabaseErrorKind::UniqueViolation
                | DatabaseErrorKind::ForeignKeyViolation
                | DatabaseErrorKind::RestrictViolation
                | DatabaseErrorKind::NotNullViolation
                | DatabaseErrorKind::CheckViolation
                | DatabaseErrorKind::ExclusionViolation,
                info,
            ) => Self {
                error_type: Source::Database(DatabaseError::ConstraintViolation),
                message: info.message().into(),
            },

            DieselErr::DatabaseError(DatabaseErrorKind::ClosedConnection, err) => Self {
                error_type: Source::Database(DatabaseError::Connection),
                message: format!("Database closed the connection: {err:?}"),
            },

            e => Self {
                error_type: Source::Database(DatabaseError::Unknown),
                message: format!(
                    "Something unexpected happened while attempting to communicate with the database: {e:?}"
                ),
            },
        }
    }
}

// TODO: Redo this later if it's possible
// pub trait FlattenErr<T> {
//     /// Flattens nested errors caused by using the connection pool
//     ///
//     /// # Errors
//     /// Forwards whatever error was emmited either by the database or pool, flattened into an Exn
//     /// error.
//     fn flatten_err(self) -> exn::Result<T, Error>;
// }
//
// impl<T> FlattenErr<T> for Result<Result<T, diesel::result::Error>, deadpool_diesel::InteractError> {
//     fn flatten_err(self) -> exn::Result<T, Error> {
//         self.map_err(Error::from)?
//             .or_raise(|| Error::upstream("Failed to execute query".into()))
//     }
// }

#[derive(Debug)]
pub struct ErrorResponse(Exn<Error>);

impl ErrorResponse {
    #[must_use]
    pub fn insufficient_permissions() -> Self {
        Self(exn::Exn::new(Error::authentication(
            AuthenticationError::InsufficientPermissions,
            "This user is not authorized to do this operation".into(),
        )))
    }
}

impl std::convert::From<exn::Exn<Error>> for ErrorResponse {
    fn from(value: Exn<Error>) -> Self {
        Self(value)
    }
}

impl IntoResponse for ErrorResponse {
    fn into_response(self) -> axum::response::Response {
        // The root cause is the one that's propagated to the caller
        let mut lowest_level = self.0.frame();

        while let Some(lower_level) = lowest_level.children().first() {
            lowest_level = lower_level;
        }

        #[allow(clippy::option_if_let_else)]
        let error: &Error = match lowest_level.error().downcast_ref() {
            Some(error) => error,
            None => &Error::upstream("Failed to downcast error. This should never happen".into()),
        };

        let http_code = match error.error_type {
            // Authentication errors
            Source::Authentication(AuthenticationError::Unknown) => {
                tracing::warn!("{error:?}");
                StatusCode::INTERNAL_SERVER_ERROR
            }
            Source::Authentication(_) => StatusCode::UNAUTHORIZED,

            // Database errors
            Source::Database(DatabaseError::Timeout | DatabaseError::Connection) => {
                tracing::warn!("{error:?}");
                StatusCode::GATEWAY_TIMEOUT
            }
            Source::Database(DatabaseError::NotFound) => StatusCode::NOT_FOUND,
            Source::Database(DatabaseError::ConstraintViolation) => StatusCode::PRECONDITION_FAILED,

            // Anything else
            _ => {
                tracing::warn!("{error:?}");
                StatusCode::INTERNAL_SERVER_ERROR
            }
        };

        (http_code, Json(error)).into_response()
    }
}
