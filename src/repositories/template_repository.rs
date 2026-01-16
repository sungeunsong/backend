use crate::domain::template::{CreateTemplateDto, Template};
use sqlx::PgPool;
use uuid::Uuid;

pub struct TemplateRepository {
    pool: PgPool,
}

impl TemplateRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create(&self, dto: CreateTemplateDto) -> Result<Template, sqlx::Error> {
        let template = sqlx::query_as::<_, Template>(
            r#"
            INSERT INTO templates (id, name, description, form_schema, workflow_snapshot, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, NOW(), NOW())
            RETURNING id, name, description, form_schema, workflow_snapshot, created_at, updated_at
            "#
        )
        .bind(Uuid::new_v4())
        .bind(dto.name)
        .bind(dto.description)
        .bind(sqlx::types::Json(dto.form_schema))
        .bind(sqlx::types::Json(dto.workflow_snapshot))
        .fetch_one(&self.pool)
        .await?;

        Ok(template)
    }

    pub async fn find_all(&self) -> Result<Vec<Template>, sqlx::Error> {
        let templates = sqlx::query_as::<_, Template>(
            r#"
            SELECT id, name, description, form_schema, workflow_snapshot, created_at, updated_at
            FROM templates
            ORDER BY created_at DESC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(templates)
    }

    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<Template>, sqlx::Error> {
        let template = sqlx::query_as::<_, Template>(
            r#"
            SELECT id, name, description, form_schema, workflow_snapshot, created_at, updated_at
            FROM templates
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(template)
    }
}
