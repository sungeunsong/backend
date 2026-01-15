use crate::domain::approval::{ApprovalRequest, FlowProcess};
use sqlx::types::Json;
use sqlx::{PgPool, Result};
use uuid::Uuid;

pub struct ApprovalRepository {
    pool: PgPool,
}

impl ApprovalRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    // [Rust Guide]
    // async fn: 비동기 함수입니다. DB I/O 같은 느린 작업은 비동기로 처리하여 서버 성능을 높입니다.
    // Result<ApprovalRequest>: 성공하면 ApprovalRequest를 반환하고, 실패하면 Error를 반환합니다.
    pub async fn create(
        &self,
        title: String,
        requester_id: Uuid,
        form_data: serde_json::Value,
        flow_process: FlowProcess,
    ) -> Result<ApprovalRequest> {
        // [Fix] sqlx::query_as!에서 JSONB 타입을 Rust 구조체로 매핑할 때 정확한 타입을 명시해야 합니다.
        // as "field: Type" 문법을 사용합니다.
        let request = sqlx::query_as!(
            ApprovalRequest,
            r#"
            INSERT INTO pxm_approval_requests (title, requester_id, form_data, flow_process)
            VALUES ($1, $2, $3, $4)
            RETURNING
                id,
                title,
                requester_id,
                status,
                form_data as "form_data: Json<serde_json::Value>",
                flow_process as "flow_process: Json<FlowProcess>",
                created_at,
                updated_at
            "#,
            title,
            requester_id,
            Json(form_data) as Json<serde_json::Value>, // 명시적 캐스팅
            Json(flow_process) as Json<FlowProcess>
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(request)
    }

    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<ApprovalRequest>> {
        // [Rust Guide]
        // query_as!: 컴파일 타임에 SQL 문법과 타입 정합성을 검사해주는 매크로입니다.
        // 런타임 에러를 획기적으로 줄여줍니다.
        let request = sqlx::query_as!(
            ApprovalRequest,
            r#"
            SELECT
                id,
                title,
                requester_id,
                status,
                form_data as "form_data: Json<serde_json::Value>",
                flow_process as "flow_process: Json<FlowProcess>",
                created_at,
                updated_at
            FROM pxm_approval_requests
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(request)
    }
}
