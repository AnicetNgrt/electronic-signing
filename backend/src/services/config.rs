use anyhow::{Context, Result};
use std::env;

#[derive(Clone, Debug)]
pub struct Config {
    pub database_url: String,
    pub backend_host: String,
    pub backend_port: u16,
    pub jwt_secret: String,
    pub jwt_expiration_hours: i64,
    pub bcrypt_cost: u32,
    pub admin_email: String,
    pub admin_password: String,
    pub smtp_host: String,
    pub smtp_port: u16,
    pub smtp_username: String,
    pub smtp_password: String,
    pub smtp_from_email: String,
    pub smtp_from_name: String,
    pub smtp_tls: bool,
    pub storage_path: String,
    pub max_file_size_mb: u64,
    pub hash_algorithm: String,
    pub public_url: String,
    pub rate_limit_rpm: u32,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            database_url: env::var("DATABASE_URL").context("DATABASE_URL must be set")?,
            backend_host: env::var("BACKEND_HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            backend_port: env::var("BACKEND_PORT")
                .unwrap_or_else(|_| "8080".to_string())
                .parse()
                .context("BACKEND_PORT must be a valid port number")?,
            jwt_secret: env::var("JWT_SECRET").context("JWT_SECRET must be set")?,
            jwt_expiration_hours: env::var("JWT_EXPIRATION_HOURS")
                .unwrap_or_else(|_| "24".to_string())
                .parse()
                .context("JWT_EXPIRATION_HOURS must be a number")?,
            bcrypt_cost: env::var("BCRYPT_COST")
                .unwrap_or_else(|_| "12".to_string())
                .parse()
                .context("BCRYPT_COST must be a number")?,
            admin_email: env::var("ADMIN_EMAIL").context("ADMIN_EMAIL must be set")?,
            admin_password: env::var("ADMIN_PASSWORD").context("ADMIN_PASSWORD must be set")?,
            smtp_host: env::var("SMTP_HOST").unwrap_or_else(|_| "localhost".to_string()),
            smtp_port: env::var("SMTP_PORT")
                .unwrap_or_else(|_| "587".to_string())
                .parse()
                .context("SMTP_PORT must be a valid port number")?,
            smtp_username: env::var("SMTP_USERNAME").unwrap_or_default(),
            smtp_password: env::var("SMTP_PASSWORD").unwrap_or_default(),
            smtp_from_email: env::var("SMTP_FROM_EMAIL")
                .unwrap_or_else(|_| "noreply@localhost".to_string()),
            smtp_from_name: env::var("SMTP_FROM_NAME").unwrap_or_else(|_| "SignVault".to_string()),
            smtp_tls: env::var("SMTP_TLS")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true),
            storage_path: env::var("STORAGE_PATH").unwrap_or_else(|_| "./data/storage".to_string()),
            max_file_size_mb: env::var("MAX_FILE_SIZE_MB")
                .unwrap_or_else(|_| "50".to_string())
                .parse()
                .context("MAX_FILE_SIZE_MB must be a number")?,
            hash_algorithm: env::var("HASH_ALGORITHM").unwrap_or_else(|_| "SHA256".to_string()),
            public_url: env::var("PUBLIC_URL")
                .unwrap_or_else(|_| "http://localhost:5173".to_string()),
            rate_limit_rpm: env::var("RATE_LIMIT_RPM")
                .unwrap_or_else(|_| "60".to_string())
                .parse()
                .context("RATE_LIMIT_RPM must be a number")?,
        })
    }

    pub fn max_file_size_bytes(&self) -> u64 {
        self.max_file_size_mb * 1024 * 1024
    }
}
