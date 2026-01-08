use sqlx::PgPool;
use std::sync::Arc;

use crate::services::config::Config;
use crate::services::email::EmailService;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub config: Config,
    pub email_service: Option<Arc<EmailService>>,
}

impl AppState {
    pub fn new(pool: PgPool, config: Config) -> Self {
        let email_service = crate::services::email::create_email_service(&config)
            .ok()
            .flatten()
            .map(Arc::new);

        Self {
            pool,
            config,
            email_service,
        }
    }
}
