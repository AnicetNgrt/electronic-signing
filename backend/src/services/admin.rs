use anyhow::Result;
use sqlx::PgPool;
use tracing::info;

use crate::db;
use crate::services::config::Config;

pub async fn ensure_admin_exists(pool: &PgPool, config: &Config) -> Result<()> {
    let admin_count = db::user::count_admin_users(pool).await?;

    if admin_count == 0 {
        info!(
            "No admin user found, creating initial admin: {}",
            config.admin_email
        );

        let password_hash = bcrypt::hash(&config.admin_password, config.bcrypt_cost)?;

        db::user::create_user(
            pool,
            &config.admin_email,
            &password_hash,
            "Administrator",
            true,
        )
        .await?;

        info!("Initial admin user created successfully");
    } else {
        info!("Admin user(s) already exist, skipping creation");
    }

    Ok(())
}
