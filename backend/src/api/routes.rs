use axum::{
    extract::State,
    middleware,
    routing::{delete, get, post, put},
    Json, Router,
};
use serde::Serialize;

use crate::api::{auth, documents, middleware::auth_middleware, signing, state::AppState};

pub fn create_routes(state: AppState) -> Router {
    let public_routes = Router::new()
        .route("/health", get(health_check))
        .route("/health/detailed", get(detailed_health_check))
        .route("/auth/login", post(auth::login));

    let signing_routes = Router::new()
        .route("/sign/:token", get(signing::get_signing_session))
        .route("/sign/:token/pdf", get(signing::get_signing_pdf))
        .route("/sign/:token/submit", post(signing::submit_signing))
        .route(
            "/sign/:token/decline",
            post(signing::decline_signing_request),
        );

    let protected_routes = Router::new()
        .route("/auth/me", get(auth::get_current_user))
        .route("/documents", get(documents::list_documents))
        .route("/documents", post(documents::create_document))
        .route("/documents/:id", get(documents::get_document))
        .route("/documents/:id", delete(documents::delete_document))
        .route("/documents/:id/fields", post(documents::add_field))
        .route(
            "/documents/:id/fields/:field_id",
            put(documents::update_field),
        )
        .route(
            "/documents/:id/fields/:field_id",
            delete(documents::delete_field),
        )
        .route("/documents/:id/signers", post(documents::add_signer))
        .route(
            "/documents/:id/signers/:signer_id",
            delete(documents::remove_signer),
        )
        .route("/documents/:id/send", post(documents::send_document))
        .route("/documents/:id/void", post(documents::void_document))
        .route("/documents/:id/audit", get(documents::get_audit_logs))
        .route(
            "/documents/:id/certificate",
            get(documents::get_certificate),
        )
        .route("/documents/:id/download", get(documents::download_document))
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            auth_middleware,
        ));

    Router::new()
        .merge(public_routes)
        .merge(signing_routes)
        .merge(protected_routes)
        .with_state(state)
}

async fn health_check() -> &'static str {
    "OK"
}

#[derive(Serialize)]
struct HealthStatus {
    status: String,
    version: String,
    database: DatabaseHealth,
    storage: StorageHealth,
}

#[derive(Serialize)]
struct DatabaseHealth {
    connected: bool,
    latency_ms: Option<u64>,
    error: Option<String>,
}

#[derive(Serialize)]
struct StorageHealth {
    writable: bool,
    path: String,
    error: Option<String>,
}

async fn detailed_health_check(State(state): State<AppState>) -> Json<HealthStatus> {
    // Check database connectivity
    let db_start = std::time::Instant::now();
    let db_result: Result<_, sqlx::Error> = sqlx::query("SELECT 1").execute(&state.pool).await;
    let db_health = match db_result {
        Ok(_) => DatabaseHealth {
            connected: true,
            latency_ms: Some(db_start.elapsed().as_millis() as u64),
            error: None,
        },
        Err(e) => DatabaseHealth {
            connected: false,
            latency_ms: None,
            error: Some(e.to_string()),
        },
    };

    // Check storage directory
    let storage_path = &state.config.storage_path;
    let storage_health = if std::path::Path::new(storage_path).exists() {
        // Try to write a test file
        let test_file = format!("{}/.health_check", storage_path);
        match std::fs::write(&test_file, "test") {
            Ok(_) => {
                let _ = std::fs::remove_file(&test_file);
                StorageHealth {
                    writable: true,
                    path: storage_path.clone(),
                    error: None,
                }
            }
            Err(e) => StorageHealth {
                writable: false,
                path: storage_path.clone(),
                error: Some(e.to_string()),
            },
        }
    } else {
        StorageHealth {
            writable: false,
            path: storage_path.clone(),
            error: Some("Storage directory does not exist".to_string()),
        }
    };

    let overall_status = if db_health.connected && storage_health.writable {
        "healthy"
    } else {
        "unhealthy"
    };

    Json(HealthStatus {
        status: overall_status.to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        database: db_health,
        storage: storage_health,
    })
}
