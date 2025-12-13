use sqlx::{PgPool};
use uuid::Uuid;
use crate::models::submission::{
    submission_unit::SubmissionUnit,
    document::Document,
    context_of_use::ContextOfUse,
    // keyword_definition::KeywordDefinition
};
use crate::error::{Result, Error};

pub struct SubmissionRepository {
    pool: PgPool,
}

impl SubmissionRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Persists a full SubmissionUnit
    pub async fn create_submission(&self, unit: &SubmissionUnit) -> Result<Uuid> {
        let mut tx = self.pool.begin().await.map_err(|e| Error::Database(e.to_string()))?;

        // 1. Insert Unit
        sqlx::query(
            r#"
            INSERT INTO submission_units
            (id, submission_id, sequence_number, code, code_system, status_code)
            VALUES ($1, $2, $3, $4, $5, $6)
            "#
        )
        .bind(unit.id)
        .bind(unit.submission_id)
        .bind(unit.sequence_number)
        .bind(&unit.code)
        .bind(&unit.code_system)
        .bind(&unit.status_code)
        .execute(&mut *tx)
        .await.map_err(|e| Error::Database(e.to_string()))?;

        // 2. Insert Documents (Wait, we need to handle the relationship)
        // For this refactor, we assume documents are inserted via `add_document_to_submission`
        // or we loop here if 'unit' carries them.

        tx.commit().await.map_err(|e| Error::Database(e.to_string()))?;
        Ok(unit.id)
    }

    /// Adds a document to a submission
    pub async fn add_document_to_submission(
        &self,
        unit_id: Uuid,
        doc: &Document,
        cou: &ContextOfUse,
    ) -> Result<()> {
        let mut tx = self.pool.begin().await.map_err(|e| Error::Database(e.to_string()))?;

        // 1. Insert Document
        sqlx::query(
            r#"
            INSERT INTO documents
            (id, submission_unit_id, xlink_href, checksum, checksum_algorithm, title)
            VALUES ($1, $2, $3, $4, $5, $6)
            "#
        )
        .bind(doc.id)
        .bind(unit_id)
        .bind(&doc.xlink_href)
        .bind(&doc.checksum)
        .bind(&doc.checksum_algorithm)
        .bind(&doc.title)
        .execute(&mut *tx)
        .await.map_err(|e| Error::Database(e.to_string()))?;

        // 2. Insert Context of Use
        sqlx::query(
            r#"
            INSERT INTO contexts_of_use
            (id, submission_unit_id, code, code_system, status_code, priority_number, document_reference_id)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#
        )
        .bind(cou.id)
        .bind(unit_id)
        .bind(&cou.code)
        .bind(&cou.code_system)
        .bind(&cou.status_code)
        .bind(cou.priority_number)
        .bind(cou.document_reference_id)
        .execute(&mut *tx)
        .await.map_err(|e| Error::Database(e.to_string()))?;

        tx.commit().await.map_err(|e| Error::Database(e.to_string()))?;
        Ok(())
    }
}
