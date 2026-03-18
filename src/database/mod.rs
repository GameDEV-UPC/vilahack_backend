use deadpool_diesel::{Manager, Pool as ConnectionPool, Runtime, postgres::Connection};
use diesel::PgConnection;

use crate::error::Error;
use exn::ResultExt;

const MAX_POOL_SIZE: usize = 8;
pub struct Pool(ConnectionPool<Manager<PgConnection>>);

pub mod schema;

impl Pool {
    /// Builds a `PostgreSQL` connection pool from the provided database url
    /// (formatted as a [connection string](https://www.postgresql.org/docs/9.4/libpq-connect.html#LIBPQ-CONNSTRING))
    /// with `MAX_POOL_SIZE`.
    ///
    /// # Errors
    /// An error will be returned if the database url is malformed or if a connection cannot
    /// be established.
    ///
    /// # Panics
    /// Never, the unwrap is for an infallible operation
    pub async fn from_url(url: &str) -> exn::Result<Self, Error> {
        let manager = Manager::new(url, Runtime::Tokio1);

        // Infallible!
        let pool = ConnectionPool::builder(manager)
            .max_size(MAX_POOL_SIZE)
            .build()
            .unwrap();

        // Check if pool is usable by attempting to get a connection
        let _ = pool.get().await.map_err(Error::from).or_raise(|| {
            Error::upstream("Failed to get a test connection while building the pool".into())
        })?;

        Ok(Self(pool))
    }

    /// Retrieves a connection from the pool
    ///
    /// # Errors
    /// Will return an error if there is a timeout or a connection error trying to retrieve a connection from the pool
    pub async fn get(&self) -> exn::Result<Connection, Error> {
        self.0.get().await.map_err(Error::from).or_raise(|| {
            tracing::warn!(target: "database", err = "pool_fetch");
            Error::upstream("Failed to get a connection from the pool".into())
        })
    }
}
