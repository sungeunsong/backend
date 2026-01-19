use crate::{
    domain::approval::{ApprovalAction, FlowProcess},
    repositories::approval_repository::ApprovalRepository,
};
use axum::{
    Json,
    extract::{Extension, Path, State},
    http::StatusCode,
};
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct CreateApprovalRequestDto {
    pub title: String,
    // requester_id is removed from DTO, will use Auth
    pub form_data: serde_json::Value,
    pub flow_process: FlowProcess,
}

pub async fn create_approval(
    State(pool): State<PgPool>,
    Extension(user_id): Extension<Uuid>,
    Json(payload): Json<CreateApprovalRequestDto>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let repo = ApprovalRepository::new(pool);

    let request = repo
        .create(
            payload.title,
            user_id, // Use authenticated user
            payload.form_data,
            payload.flow_process,
        )
        .await
        .map_err(|e| {
            eprintln!("Failed to create approval request: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    // Log creation
    let _ = repo
        .add_log(request.id, user_id, "CREATED".to_string(), None)
        .await;

    Ok(Json(serde_json::json!(request)))
}

pub async fn get_approval(
    Path(id): Path<Uuid>,
    State(pool): State<PgPool>,
    Extension(_user_id): Extension<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let repo = ApprovalRepository::new(pool);

    match repo.find_by_id(id).await {
        Ok(Some(request)) => {
            // TODO: Use user_id to filter permissions if needed (e.g. only involved people can view)
            Ok(Json(serde_json::json!(request)))
        }
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            eprintln!("Failed to get approval request: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// Handler for APPROVE
pub async fn approve_request(
    Path(id): Path<Uuid>,
    State(pool): State<PgPool>,
    Extension(user_id): Extension<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    process_action_internal(id, ApprovalAction::Approve, None, pool, user_id).await
}

// Handler for REJECT
#[derive(Deserialize)]
pub struct RejectDto {
    pub reason: String,
}

pub async fn reject_request(
    Path(id): Path<Uuid>,
    State(pool): State<PgPool>,
    Extension(user_id): Extension<Uuid>,
    Json(payload): Json<RejectDto>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    process_action_internal(
        id,
        ApprovalAction::Reject,
        Some(payload.reason),
        pool,
        user_id,
    )
    .await
}

// Internal logic shared by both
async fn process_action_internal(
    id: Uuid,
    action: ApprovalAction,
    reason: Option<String>,
    pool: PgPool,
    actor_id: Uuid,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let repo = ApprovalRepository::new(pool);

    // 1. Fetch
    let mut request = repo
        .find_by_id(id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::NOT_FOUND, "Request not found".to_string()))?;

    // Validate that actor_id matches the current approver
    let current_step_idx = (request.flow_process.0.current_step - 1) as usize;
    let expected_approver_id = match request.flow_process.0.steps.get(current_step_idx) {
        Some(step) => step.approver_id,
        None => return Err((StatusCode::BAD_REQUEST, "Invalid step state".to_string())),
    };

    if expected_approver_id != actor_id {
        return Err((
            StatusCode::FORBIDDEN,
            "You are not the current approver".to_string(),
        ));
    }

    // 2. Handle Action via Domain Logic
    let result_msg = request
        .flow_process
        .0
        .handle_action(action, expected_approver_id)
        .map_err(|e| (StatusCode::BAD_REQUEST, e))?;

    if result_msg == "completed" {
        request.status = "approved".to_string();
    } else if result_msg == "rejected" {
        request.status = "rejected".to_string();
    }

    // 3. Update DB
    let updated = repo
        .update(request)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // 4. Log
    let log_action = match action {
        ApprovalAction::Approve => "APPROVED",
        ApprovalAction::Reject => "REJECTED",
    };
    let _ = repo
        .add_log(id, actor_id, log_action.to_string(), reason)
        .await;

    Ok(Json(serde_json::json!(updated)))
}

#[derive(Deserialize)]
pub struct CommentDto {
    pub content: String,
}

pub async fn add_comment(
    Path(id): Path<Uuid>,
    State(pool): State<PgPool>,
    Extension(user_id): Extension<Uuid>,
    Json(payload): Json<CommentDto>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let repo = ApprovalRepository::new(pool);

    // Ensure request exists
    let _request = repo
        .find_by_id(id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::NOT_FOUND, "Request not found".to_string()))?;

    // Log comment
    let log = repo
        .add_log(
            id,
            user_id, // Use authenticated user
            "COMMENT".to_string(),
            Some(payload.content),
        )
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(serde_json::json!(log)))
}

pub async fn get_logs(
    Path(id): Path<Uuid>,
    State(pool): State<PgPool>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let repo = ApprovalRepository::new(pool);
    match repo.get_logs(id).await {
        Ok(logs) => Ok(Json(serde_json::json!(logs))),
        Err(e) => {
            eprintln!("Failed to fetch logs: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn list_approvals(
    State(pool): State<PgPool>,
    Extension(user_id): Extension<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let repo = ApprovalRepository::new(pool);

    // TODO: Filter by user_id (My Inbox)
    // For now, list all but we should filter where user is requester OR approver
    let _ = user_id;
    match repo.find_all().await {
        Ok(requests) => Ok(Json(serde_json::json!(requests))),
        Err(e) => {
            eprintln!("Failed to list approvals: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
