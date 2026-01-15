use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::types::Json;
use uuid::Uuid;

// [Rust Guide]
// JSONB 데이터를 다루기 위한 구조체입니다.
// Serialize, Deserialize: Rust 구조체 <-> JSON 변환을 위해 필요합니다.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FlowProcess {
    pub current_step: i32,
    pub steps: Vec<ApprovalStep>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ApprovalStep {
    pub seq: i32,
    pub approver_id: Uuid,
    pub status: String, // pending, approved, rejected
    pub timestamp: Option<DateTime<Utc>>,
}

// [Rust Guide]
// sqlx::FromRow: DB 조회 결과를 이 구조체에 자동으로 매핑해줍니다.
// Json<T>: PostgreSQL의 JSONB 컬럼을 Rust의 타입(T)으로 자동 변환해주는 래퍼(Wrapper)입니다.
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct ApprovalRequest {
    pub id: Uuid,
    pub title: String,
    pub requester_id: Uuid,
    pub status: String,

    // [Hybrid Schema Key Point]
    // DB에는 JSONB로 저장되지만, Rust 코드에서는 타입이 명확한 구조체(FlowProcess)로 다룹니다.
    pub flow_process: Json<FlowProcess>,

    // 폼 데이터는 구조가 가변적이므로 serde_json::Value를 사용하여 유연하게 처리합니다.
    pub form_data: Json<serde_json::Value>,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
