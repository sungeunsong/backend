use crate::{
    domain::{approval::FlowProcess, template::CreateTemplateDto},
    repositories::{
        approval_repository::ApprovalRepository, template_repository::TemplateRepository,
    },
};
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use sqlx::PgPool;
use uuid::Uuid;

pub async fn create_template(
    State(pool): State<PgPool>,
    Json(payload): Json<CreateTemplateDto>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let repo = TemplateRepository::new(pool);

    match repo.create(payload).await {
        Ok(template) => Ok(Json(serde_json::json!(template))),
        Err(e) => {
            eprintln!("Failed to create template: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn list_templates(
    State(pool): State<PgPool>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let repo = TemplateRepository::new(pool);

    match repo.find_all().await {
        Ok(templates) => Ok(Json(serde_json::json!(templates))),
        Err(e) => {
            eprintln!("Failed to list templates: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn get_template(
    Path(id): Path<Uuid>,
    State(pool): State<PgPool>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let repo = TemplateRepository::new(pool);

    match repo.find_by_id(id).await {
        Ok(Some(template)) => Ok(Json(serde_json::json!(template))),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            eprintln!("Failed to get template: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// 템플릿 기반으로 결재 요청 생성
// POST /approvals/from-template/:template_id
// Body: { "requester_id": "...", "form_data": { ... } }
// Title은 템플릿 이름 + Timestamp 등으로 자동 생성하거나 Body에서 받을 수도 있음.
// FlowProcess는 템플릿에 정의된 스냅샷을 복사해옴.
#[derive(serde::Deserialize)]
pub struct CreateFromTemplateDto {
    pub requester_id: Uuid,
    pub form_data: serde_json::Value,
    // title은 선택적(Option), 없으면 "{Template Name} - {Date}" 형식
    pub title: Option<String>,
}

pub async fn create_approval_from_template(
    Path(template_id): Path<Uuid>,
    State(pool): State<PgPool>,
    Json(payload): Json<CreateFromTemplateDto>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let template_repo = TemplateRepository::new(pool.clone());
    let approval_repo = ApprovalRepository::new(pool);

    // 1. 템플릿 조회
    let template = match template_repo.find_by_id(template_id).await {
        Ok(Some(t)) => t,
        Ok(None) => return Err((StatusCode::NOT_FOUND, "Template not found".to_string())),
        Err(e) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("DB Error: {:?}", e),
            ));
        }
    };

    // 2. 제목 생성
    let title = payload.title.unwrap_or_else(|| {
        format!(
            "{} - {}",
            template.name,
            chrono::Utc::now().format("%Y-%m-%d %H:%M")
        )
    });

    // 3. 결재 요청 생성 (템플릿의 워크플로우 복사)
    // 주의: 실제로는 form_data가 form_schema에 맞는지 검증(Validation)하는 로직이 필요함.

    // Convert Json<FlowProcess> to FlowProcess
    let flow_process = template.workflow_snapshot.0;

    match approval_repo
        .create(title, payload.requester_id, payload.form_data, flow_process)
        .await
    {
        Ok(request) => Ok(Json(serde_json::json!(request))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to create approval: {:?}", e),
        )),
    }
}
