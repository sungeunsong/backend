use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::types::Json;
use uuid::Uuid;

// [Rust Guide]
// JSONB 데이터를 다루기 위한 구조체입니다.
// Serialize, Deserialize: Rust 구조체 <-> JSON 변환을 위해 필요합니다.

#[derive(Debug, Deserialize, Clone, Copy, PartialEq)]
#[serde(rename_all = "lowercase")] // JSON: "approve", "reject"
pub enum ApprovalAction {
    Approve,
    Reject,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FlowProcess {
    pub current_step: i32,
    pub steps: Vec<ApprovalStep>,
}

impl FlowProcess {
    pub fn handle_action(
        &mut self,
        action: ApprovalAction,
        approver_id: Uuid,
    ) -> Result<String, String> {
        // 1. 현재 단계 찾기
        // get_mut: 가변 참조를 가져옵니다. (데이터 수정 권한)
        let step_idx = (self.current_step - 1) as usize;
        let step = self
            .steps
            .get_mut(step_idx)
            .ok_or("Current step not found")?;

        // 2. 권한 확인 (본인 차례인지)
        if step.approver_id != approver_id {
            return Err("Not your turn to approve".to_string());
        }

        // 3. 이미 처리되었는지 확인
        if step.status != "pending" {
            return Err("Already processed".to_string());
        }

        // 4. 상태 업데이트
        match action {
            ApprovalAction::Approve => {
                step.status = "approved".to_string();
                step.timestamp = Some(Utc::now());

                // 다음 단계로 이동 확인
                if self.current_step < self.steps.len() as i32 {
                    self.current_step += 1;
                    return Ok("moved_to_next_step".to_string());
                } else {
                    return Ok("completed".to_string());
                }
            }
            ApprovalAction::Reject => {
                step.status = "rejected".to_string();
                step.timestamp = Some(Utc::now());
                return Ok("rejected".to_string());
            }
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ApprovalStep {
    pub seq: i32,
    pub name: String, // Step name (e.g. "Manager Approval")
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

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct ApprovalLog {
    pub id: Uuid,
    pub approval_id: Uuid,
    pub actor_id: Uuid,
    pub action_type: String,
    pub content: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
}
