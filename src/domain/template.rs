use super::approval::FlowProcess;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::types::Json;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Template {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,

    // 폼의 구조를 정의합니다 (예: 필드명, 타입 등) - UI 렌더링용
    pub form_schema: Json<serde_json::Value>,

    // 이 템플릿으로 생성될 때 기본적으로 적용될 결재선
    pub workflow_snapshot: Json<FlowProcess>,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateTemplateDto {
    pub name: String,
    pub description: Option<String>,
    pub form_schema: serde_json::Value,
    pub workflow_snapshot: FlowProcess,
}
