pub mod api;
pub mod authentication;
pub mod config;
pub mod database;
pub mod discrimination;
pub mod error;
pub mod model;
pub mod telemetry;

use std::{collections::HashMap, path::PathBuf, sync::Mutex};
use tokio::task::JoinHandle;

use deadpool_diesel::postgres::Connection;
use uuid::Uuid;

use crate::{config::CONFIG, database::Pool, error::Error};

pub type Taskmap = HashMap<(Uuid, Uuid), JoinHandle<exn::Result<PathBuf, Error>>>;

pub struct State {
    pool: Pool,
    pub tasks: Mutex<Taskmap>,
}

impl State {
    /// # Panics
    /// If the database url is not reachable
    pub async fn new() -> Self {
        Self {
            pool: database::Pool::from_url(&CONFIG.database_url)
                .await
                .expect("Connection pool could not be built"),
            tasks: Mutex::new(Taskmap::new()),
        }
    }

    /// Retrieves a connection from the pool
    ///
    /// # Errors
    /// Will return an error if there is a timeout or a connection error trying to retrieve a connection from the pool
    pub async fn get_connection(&self) -> exn::Result<Connection, Error> {
        self.pool.get().await
    }
}
