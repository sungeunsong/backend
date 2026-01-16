use crate::{
    domain::approval::{ApprovalAction, FlowProcess},
    repositories::approval_repository::ApprovalRepository,
};
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;

// [Rust Guide]
// DTO (Data Transfer Object): 클라이언트에서 보내는 JSON 데이터를 받기 위한 구조체입니다.
// 도메인 엔티티(ApprovalRequest)와 분리하여, API 요청에 특화된 형태를 가집니다.
#[derive(Deserialize)]
pub struct CreateApprovalRequestDto {
    pub title: String,
    pub requester_id: Uuid,
    pub form_data: serde_json::Value,
    pub flow_process: FlowProcess,
}

// [Rust Guide]
// State<PgPool>: main.rs에서 .with_state(pool)로 주입한 DB 연결 풀을 받아옵니다.
// Json(payload): 요청 바디의 JSON을 자동으로 파싱하여 DTO로 변환해줍니다.
pub async fn create_approval(
    State(pool): State<PgPool>,
    Json(payload): Json<CreateApprovalRequestDto>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // Repository 인스턴스 생성
    let repo = ApprovalRepository::new(pool);

    // DB 저장 요청
    match repo
        .create(
            payload.title,
            payload.requester_id,
            payload.form_data,
            payload.flow_process,
        )
        .await
    {
        Ok(request) => {
            // 성공 시 생성된 객체를 JSON으로 반환
            // serde_json::json! 매크로를 사용하여 간편하게 JSON 생성
            Ok(Json(serde_json::json!(request)))
        }
        Err(e) => {
            // 에러 로깅 (실무에서는 tracing crate 사용 권장)
            eprintln!("Failed to create approval request: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn get_approval(
    Path(id): axum::extract::Path<Uuid>,
    State(pool): State<PgPool>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let repo = ApprovalRepository::new(pool);

    match repo.find_by_id(id).await {
        Ok(Some(request)) => Ok(Json(serde_json::json!(request))),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            eprintln!("Failed to get approval request: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

#[derive(Deserialize)]
pub struct ProcessApprovalDto {
    pub approver_id: Uuid,
    pub action: ApprovalAction,
}

pub async fn process_approval(
    Path(id): Path<Uuid>,
    State(pool): State<PgPool>,
    Json(payload): Json<ProcessApprovalDto>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let repo = ApprovalRepository::new(pool);

    // 1. Fetch
    let mut request = match repo.find_by_id(id).await {
        Ok(Some(req)) => req,
        Ok(None) => return Err((StatusCode::NOT_FOUND, "Request not found".to_string())),
        Err(e) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("DB Error: {:?}", e),
            ));
        }
    };

    // 2. Logic
    match request
        .flow_process
        .0
        .handle_action(payload.action, payload.approver_id)
    {
        Ok(result) => {
            if result == "completed" {
                request.status = "approved".to_string();
            } else if result == "rejected" {
                request.status = "rejected".to_string();
            }
            // "moved_to_next_step" -> status remains "pending"
        }
        Err(e) => return Err((StatusCode::BAD_REQUEST, e)),
    }

    // 3. Update
    match repo.update(request).await {
        Ok(updated) => Ok(Json(serde_json::json!(updated))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Update failed: {:?}", e),
        )),
    }
}

pub async fn list_approvals(
    State(pool): State<PgPool>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let repo = ApprovalRepository::new(pool);

    match repo.find_all().await {
        Ok(requests) => Ok(Json(serde_json::json!(requests))),
        Err(e) => {
            eprintln!("Failed to list approvals: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
