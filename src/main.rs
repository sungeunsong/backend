use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, postgres::PgPoolOptions};
use std::net::SocketAddr;
use tower_http::cors::CorsLayer;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UserProfile {
    pub name: String,
    pub age: i32,
    pub preferences: serde_json::Value, // 동적인 JSON 데이터를 담는 타입
}

#[derive(Serialize, Deserialize, Debug, sqlx::FromRow)]
pub struct User {
    pub id: i32,
    // DB의 JSONB를 Rust 구조체로 자동 변환해주는 어댑터
    pub profile: sqlx::types::Json<UserProfile>,
}

type AppResult<T> = Result<T, (StatusCode, String)>;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // ✅ localhost 오타 + 포트 위치 수정
    let database_url = "postgres://pxm_admin:ehdclal!!@localhost:5432/pxm_db";

    let pool: PgPool = PgPoolOptions::new()
        .max_connections(5)
        .connect(database_url)
        .await
        .expect("Failed to connect to database");

    let app = Router::new()
        .route("/users", post(create_user))
        .route("/users/{id}", get(get_user))
        .layer(CorsLayer::permissive())
        .with_state(pool);

    let addr = SocketAddr::from(([0, 0, 0, 0], 3001));
    println!("서버가 {}에서 실행 중입니다.", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn get_user(Path(id): Path<i32>, State(pool): State<PgPool>) -> AppResult<Json<User>> {
    let user = sqlx::query_as!(
        User,
        r#"SELECT id, profile as "profile: sqlx::types::Json<UserProfile>"
           FROM users
           WHERE id = $1"#,
        id
    )
    .fetch_one(&pool)
    .await
    .map_err(|e| {
        (
            StatusCode::NOT_FOUND,
            format!("유저를 찾을 수 없습니다: {}", e),
        )
    })?;

    Ok(Json(user))
}

async fn create_user(
    State(pool): State<PgPool>,
    Json(payload): Json<UserProfile>,
) -> AppResult<Json<User>> {
    let profile_value = serde_json::to_value(&payload).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("직렬화 실패: {}", e),
        )
    })?;

    let user = sqlx::query_as!(
        User,
        r#"INSERT INTO users (profile)
           VALUES ($1)
           RETURNING id, profile as "profile: sqlx::types::Json<UserProfile>""#,
        profile_value
    )
    .fetch_one(&pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("저장 실패: {}", e),
        )
    })?;

    Ok(Json(user))
}
