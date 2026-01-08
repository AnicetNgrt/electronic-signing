use axum::{
    middleware,
    routing::{delete, get, post, put},
    Router,
};

use crate::api::{auth, documents, middleware::auth_middleware, signing, state::AppState};

pub fn create_routes(state: AppState) -> Router {
    let public_routes = Router::new()
        .route("/health", get(health_check))
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
