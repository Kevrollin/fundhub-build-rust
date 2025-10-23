use serde::Deserialize;
use anyhow::Result;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub database_url: String,
    pub redis_url: String,
    pub jwt_secret: String,
    pub stellar_network: String,
    pub stellar_horizon_url: String,
    pub platform_wallet_public_key: String,
    pub platform_wallet_secret_key: String,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            database_url: std::env::var("DATABASE_URL")?,
            redis_url: std::env::var("REDIS_URL")?,
            jwt_secret: std::env::var("JWT_SECRET")?,
            stellar_network: std::env::var("STELLAR_NETWORK")?,
            stellar_horizon_url: std::env::var("STELLAR_HORIZON_URL")?,
            platform_wallet_public_key: std::env::var("PLATFORM_WALLET_PUBLIC_KEY")?,
            platform_wallet_secret_key: std::env::var("PLATFORM_WALLET_SECRET_KEY")?,
        })
    }
}

pub fn init() -> Result<Config> {
    Config::from_env()
}