// use crate::repositories::user_repository::UserRepository;
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use serde::Serialize;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Serialize)]
pub struct ManagerInfo {
    pub manager_id: Uuid,
    pub manager_name: String,
    pub department_name: String,
}

pub async fn get_my_manager(
    Path(user_id): Path<Uuid>,
    State(pool): State<PgPool>,
) -> Result<Json<ManagerInfo>, StatusCode> {
    // Logic:
    // 1. Get User -> Get Department ID
    // 2. Get Department -> Get Manager ID
    // 3. (Optional) If Manager ID is same as User (Team Lead), Get Parent Department Manager?
    //    For simple MVP: Just get direct department manager.

    // This query joins user -> department -> manager(user)
    // We can do it in one SQL query for efficiency.
    let result = sqlx::query!(
        r#"
        SELECT 
            u.id as user_id, 
            d.name as dept_name, 
            m.id as manager_id, 
            m.full_name as manager_name
        FROM users u
        JOIN departments d ON u.department_id = d.id
        JOIN users m ON d.manager_id = m.id
        WHERE u.id = $1
        "#,
        user_id
    )
    .fetch_optional(&pool)
    .await
    .map_err(|e| {
        eprintln!("DB Error: {:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    match result {
        Some(row) => Ok(Json(ManagerInfo {
            manager_id: row.manager_id,
            manager_name: row.manager_name,
            department_name: row.dept_name,
        })),
        None => Err(StatusCode::NOT_FOUND), // User has no department or department has no manager
    }
}

// Simple User Search/List
pub async fn list_users(
    State(pool): State<PgPool>,
) -> Result<Json<Vec<serde_json::Value>>, StatusCode> {
    let users = sqlx::query!(
        r#"
        SELECT id, full_name, email, position 
        FROM users 
        WHERE status = 'ACTIVE' 
        ORDER BY full_name ASC
        "#
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| {
        eprintln!("Failed to list users: {:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let dtos = users
        .into_iter()
        .map(|u| {
            serde_json::json!({
                "id": u.id,
                "full_name": u.full_name,
                "email": u.email,
                "position": u.position
            })
        })
        .collect();

    Ok(Json(dtos))
}
