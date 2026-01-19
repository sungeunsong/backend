use axum::{
    body::Body,
    extract::State,
    http::{Request, StatusCode, header},
    middleware::Next,
    response::Response,
};
use jsonwebtoken::{DecodingKey, Validation, decode};
use sqlx::PgPool;
use uuid::Uuid;

use crate::utils::auth::Claims;

// State를 사용하지 않더라도 미들웨어 시그니처를 맞추기 위해 둠 (필요 시 DB 조회 등 확장에 대비)
pub async fn auth_middleware(
    State(_pool): State<PgPool>,
    mut req: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    // 1. Authorization 헤더 추출
    let auth_header = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .ok_or(StatusCode::UNAUTHORIZED)?;

    // 2. "Bearer {token}" 형식 파싱
    if !auth_header.starts_with("Bearer ") {
        return Err(StatusCode::UNAUTHORIZED);
    }
    let token = &auth_header[7..];

    // 3. JWT 검증
    let secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "secret".to_string());
    let validation = Validation::default();

    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &validation,
    )
    .map_err(|_| StatusCode::UNAUTHORIZED)?;

    // 4. 요청에 User ID 주입 (Extension 사용)
    let user_id = Uuid::parse_str(&token_data.claims.sub).map_err(|_| StatusCode::UNAUTHORIZED)?;
    req.extensions_mut().insert(user_id);

    // 5. 다음 핸들러로 진행
    Ok(next.run(req).await)
}
