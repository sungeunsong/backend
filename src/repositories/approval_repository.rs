use crate::domain::approval::{ApprovalLog, ApprovalRequest, FlowProcess};
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

    // ... existing methods ...
    pub async fn create(
        &self,
        title: String,
        requester_id: Uuid,
        form_data: serde_json::Value,
        flow_process: FlowProcess,
    ) -> Result<ApprovalRequest> {
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
            Json(form_data) as Json<serde_json::Value>,
            Json(flow_process) as Json<FlowProcess>
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(request)
    }

    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<ApprovalRequest>> {
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

    pub async fn update(&self, request: ApprovalRequest) -> Result<ApprovalRequest> {
        let updated_request = sqlx::query_as!(
            ApprovalRequest,
            r#"
            UPDATE pxm_approval_requests
            SET
                status = $1,
                flow_process = $2,
                updated_at = NOW()
            WHERE id = $3
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
            request.status,
            request.flow_process as Json<FlowProcess>,
            request.id
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(updated_request)
    }

    pub async fn find_all(&self) -> Result<Vec<ApprovalRequest>> {
        let requests = sqlx::query_as!(
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
            ORDER BY created_at DESC
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(requests)
    }

    pub async fn add_log(
        &self,
        approval_id: Uuid,
        actor_id: Uuid,
        action_type: String,
        content: Option<String>,
    ) -> Result<ApprovalLog> {
        let log = sqlx::query_as!(
            ApprovalLog,
            r#"
            INSERT INTO approval_logs (approval_id, actor_id, action_type, content)
            VALUES ($1, $2, $3, $4)
            RETURNING *
            "#,
            approval_id,
            actor_id,
            action_type,
            content
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(log)
    }

    pub async fn get_logs(&self, approval_id: Uuid) -> Result<Vec<ApprovalLog>> {
        let logs = sqlx::query_as!(
            ApprovalLog,
            r#"
            SELECT * FROM approval_logs
            WHERE approval_id = $1
            ORDER BY created_at ASC
            "#,
            approval_id
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(logs)
    }
}
