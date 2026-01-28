use deadpool_diesel::{postgres::Connection, Manager, Pool as ConnectionPool, Runtime};
use diesel::PgConnection;

use tracing::{info, trace};

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
    pub async fn from_url(url: &str) -> Result<Self, ()> {
        let manager = Manager::new(url, Runtime::Tokio1);
        let pool = ConnectionPool::builder(manager)
            .max_size(MAX_POOL_SIZE)
            .build()
            .map_err(|_| ())?; // Infallible, runtime is specified

        _ = pool.get().await.map_err(|_| ())?; // Check if pool is usable by attempting to get a connection
        info!("Initial connection to pool successful.");

        Ok(Self(pool))
    }

    /// Retrieves a connection from the pool
    ///
    /// # Errors
    /// Will return an HTTP `SERVICE_UNAVAILABLE` error if a connection could not be retrieved
    pub async fn get(&self) -> Result<Connection, ()> {
        match self.0.get().await {
            Ok(connection) => {
                trace!("Got connectin from database pool.");
                Ok(connection)
            }

            Err(_) => {
                todo!();
            }
        }
    }
}
