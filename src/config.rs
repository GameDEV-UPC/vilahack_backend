use std::{fs, net::SocketAddr, path::PathBuf, sync::LazyLock};

use axum::http::HeaderValue;
use ipnet::IpNet;
use jsonwebtoken::jwk::JwkSet;
use serde::Deserialize;
use tracing::level_filters::LevelFilter;

pub mod headervalues {
    use axum::http::HeaderValue;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    #[allow(clippy::missing_errors_doc)]
    pub fn deserialize<'de, D>(d: D) -> Result<Vec<HeaderValue>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let strs: Vec<String> = Vec::deserialize(d)?;
        strs.into_iter()
            .map(|s| HeaderValue::from_str(&s).map_err(serde::de::Error::custom))
            .collect()
    }

    #[allow(clippy::missing_errors_doc)]
    pub fn serialize<S>(v: &[HeaderValue], s: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let strs: Vec<&str> = v.iter().map(|h| h.to_str().unwrap_or_default()).collect();
        strs.serialize(s)
    }
}

pub mod levelfilter {
    use serde::{Deserialize, Deserializer, Serializer};
    use tracing::level_filters::LevelFilter;

    #[allow(clippy::missing_errors_doc)]
    pub fn serialize<S>(lvl: &LevelFilter, s: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        s.serialize_str(&lvl.to_string())
    }

    #[allow(clippy::missing_errors_doc)]
    pub fn deserialize<'de, D>(d: D) -> Result<LevelFilter, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(d)?;
        s.parse::<LevelFilter>().map_err(serde::de::Error::custom)
    }
}

#[derive(Deserialize)]
pub struct Config {
    pub bind_address: SocketAddr,
    #[serde(with = "headervalues")]
    pub allowed_origins: Vec<HeaderValue>,

    pub database_url: String,
    pub jwk: Jwk,
    pub allowed_ranges: Vec<IpNet>,

    pub puzzle_directory: PathBuf,

    pub deployment: String,
    #[serde(with = "levelfilter")]
    pub trace_level: LevelFilter,
}

#[derive(Deserialize)]
pub struct Jwk {
    pub issuers: Vec<String>,
    pub set: JwkSet,
    pub authenticated_audiences: Vec<String>,
    pub admin_role: String,
}

pub static CONFIG: LazyLock<Config> = LazyLock::new(|| {
    let config_path = std::env::args().nth(1).map_or_else(
        || {
            tracing::info!(
                "Config path not provided, using default: /etc/vilahack_backend/config.toml"
            );
            PathBuf::from("/etc/vilahack_backend/config.toml")
        },
        PathBuf::from,
    );

    let contents = match fs::read_to_string(&config_path) {
        Ok(contents) => contents,
        Err(err) => {
            tracing::error!("Could not read the config file at {config_path:?}: {err}");
            panic!();
        }
    };

    match toml::from_str(&contents) {
        Ok(configuration) => configuration,
        Err(err) => {
            tracing::error!("Could not parse config.toml: {err}");
            panic!();
        }
    }
});
