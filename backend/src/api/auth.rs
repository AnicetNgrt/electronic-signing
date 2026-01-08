use axum::{extract::State, Extension, Json};
use chrono::Utc;
use jsonwebtoken::{encode, EncodingKey, Header};
use validator::Validate;

use crate::api::error::{ApiError, ApiResult};
use crate::api::middleware::AuthUser;
use crate::api::state::AppState;
use crate::db;
use crate::models::user::{Claims, LoginRequest, LoginResponse, UserPublic};

pub async fn login(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> ApiResult<Json<LoginResponse>> {
    req.validate()
        .map_err(|e| ApiError::Validation(e.to_string()))?;

    let user = db::user::get_user_by_email(&state.pool, &req.email)
        .await?
        .ok_or_else(|| ApiError::Unauthorized)?;

    let valid =
        bcrypt::verify(&req.password, &user.password_hash).map_err(|_| ApiError::Unauthorized)?;

    if !valid {
        return Err(ApiError::Unauthorized);
    }

    let now = Utc::now();
    let exp = now + chrono::Duration::hours(state.config.jwt_expiration_hours);

    let claims = Claims {
        sub: user.id.to_string(),
        user_id: user.id,
        email: user.email.clone(),
        is_admin: user.is_admin,
        iat: now.timestamp(),
        exp: exp.timestamp(),
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(state.config.jwt_secret.as_bytes()),
    )
    .map_err(|e| ApiError::Internal(anyhow::anyhow!("Token encoding failed: {}", e)))?;

    Ok(Json(LoginResponse {
        token,
        user: UserPublic::from(user),
    }))
}

pub async fn me(Extension(auth_user): Extension<AuthUser>) -> ApiResult<Json<UserPublic>> {
    Ok(Json(UserPublic {
        id: auth_user.user_id,
        email: auth_user.email,
        name: String::new(),
        is_admin: auth_user.is_admin,
    }))
}

pub async fn get_current_user(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
) -> ApiResult<Json<UserPublic>> {
    let user = db::user::get_user_by_id(&state.pool, auth_user.user_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("User not found".to_string()))?;

    Ok(Json(UserPublic::from(user)))
}
