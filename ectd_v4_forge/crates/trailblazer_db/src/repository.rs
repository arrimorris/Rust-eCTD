use crate::models::domain::submission_unit::SubmissionUnit;
use crate::models::entities::{DocumentEntity, SubmissionUnitEntity};
use sqlx::PgPool;
use uuid::Uuid;

pub struct SubmissionRepository {
    pool: PgPool,
}

impl SubmissionRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    // THE BRIDGE: Reads SQL Tables -> Returns Rich Domain Model
    pub async fn get_submission(&self, unit_id: Uuid) -> Result<SubmissionUnit, sqlx::Error> {
        // 1. Fetch the Flat Entity
        // Note: columns with type mismatch in Rust vs SQL (like `id` being varchar vs uuid) need careful handling.
        // In this schema, `submission_units.id` is UUID, but `documents.id` is VARCHAR.
        let _entity = sqlx::query_as!(
            SubmissionUnitEntity,
            "SELECT * FROM submission_units WHERE id = $1",
            unit_id
        )
        .fetch_one(&self.pool)
        .await?;

        // 2. Fetch Related Documents
        let _docs = sqlx::query_as!(
            DocumentEntity,
            "SELECT * FROM documents WHERE submission_unit_id = $1",
            unit_id
        )
        .fetch_all(&self.pool)
        .await?;

        // 3. TODO: Manual Mapping
        todo!("Map Entity to Domain")
    }
}
