use deadpool_diesel::postgres::PoolError;
use jsonwebtoken::errors::ErrorKind as JwtErr;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Error {
    #[serde(rename = "type")]
    error_type: Source,
    message: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Source {
    Authentication(AuthenticationError),
    Database(DatabaseError),
    Upstream,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuthenticationError {
    InvalidFormat,
    InvalidIssuer,
    InvalidAlgorythm,
    InvalidSignature,
    InvalidKey,
    InvalidClaim,
    NoMatchingKey,
    InvalidValidityDate,
    InsufficientPermissions,
    Unknown,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DatabaseError {
    Timeout,
    Connection,
    Pool,
}

impl Error {
    #[must_use]
    pub const fn authentication(error_type: AuthenticationError, message: String) -> Self {
        Self {
            error_type: Source::Authentication(error_type),
            message,
        }
    }

    #[must_use]
    pub const fn upstream(message: String) -> Self {
        Self {
            error_type: Source::Upstream,
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
                error_type: Source::Authentication(AuthenticationError::InvalidValidityDate),
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
