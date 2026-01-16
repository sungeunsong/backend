use crate::{
    domain::user::{AuthResponse, CreateUserDto, LoginDto},
    repositories::user_repository::UserRepository,
    utils::auth::{create_jwt, hash_password, verify_password},
};
use axum::{Json, extract::State, http::StatusCode};
use sqlx::PgPool;
use validator::Validate;

// Register Handler
pub async fn register(
    State(pool): State<PgPool>,
    Json(payload): Json<CreateUserDto>,
) -> Result<Json<AuthResponse>, (StatusCode, String)> {
    // 1. Validate Payload
    if let Err(e) = payload.validate() {
        return Err((StatusCode::BAD_REQUEST, format!("Validation error: {}", e)));
    }

    let repo = UserRepository::new(pool);

    // 2. Check if user exists
    if let Ok(Some(_)) = repo.find_by_email(&payload.email).await {
        return Err((StatusCode::CONFLICT, "Email already exists".to_string()));
    }

    // 3. Hash Password
    let password_hash =
        hash_password(&payload.password).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

    // 4. Create User
    let user = repo
        .create(
            payload.email,
            password_hash,
            payload.full_name,
            payload.position,
            payload.department_id,
        )
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // 5. Generate Token
    let token = create_jwt(user.id).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

    Ok(Json(AuthResponse { token, user }))
}

// Login Handler
pub async fn login(
    State(pool): State<PgPool>,
    Json(payload): Json<LoginDto>,
) -> Result<Json<AuthResponse>, (StatusCode, String)> {
    let repo = UserRepository::new(pool);

    // 1. Find User
    let user = repo
        .find_by_email(&payload.email)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::UNAUTHORIZED, "Invalid credentials".to_string()))?; // Generic error for security

    // 2. Verify Password
    if !verify_password(&user.password_hash, &payload.password) {
        return Err((StatusCode::UNAUTHORIZED, "Invalid credentials".to_string()));
    }

    // 3. Generate Token
    let token = create_jwt(user.id).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

    Ok(Json(AuthResponse { token, user }))
}
