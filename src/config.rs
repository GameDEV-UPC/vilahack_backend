use std::{fs, net::SocketAddr, sync::LazyLock};

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
    let contents = match fs::read_to_string("/etc/vilahack_backend/config.toml") {
        Ok(contents) => contents,
        Err(err) => {
            eprintln!("Could not read the config file at /etc/vilahack_backend/config.toml: {err}");
            panic!();
        }
    };

    match toml::from_str(&contents) {
        Ok(configuration) => configuration,
        Err(err) => {
            eprintln!("Could not parse config.toml: {err}");
            panic!();
        }
    }
});
