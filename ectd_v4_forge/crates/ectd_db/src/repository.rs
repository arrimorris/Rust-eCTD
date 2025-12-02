use sqlx::PgPool;
use uuid::Uuid;
// Assuming you have a `models` crate where we defined the structs previously
use ectd_core::models::submission_unit::SubmissionUnit;

pub struct SubmissionRepository {
    pool: PgPool,
}

impl SubmissionRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// The "Big Bang": Takes a full SubmissionUnit struct and persists it transactionally.
    pub async fn create_submission(&self, unit: &SubmissionUnit) -> Result<Uuid, sqlx::Error> {
        // 1. START THE TRANSACTION
        // If anything panics or errors after this, 'tx' drops and rolls back automatically.
        let mut tx = self.pool.begin().await?;

        // ---------------------------------------------------------
        // LEVEL 1: Insert the Submission Unit Container
        // ---------------------------------------------------------
        let unit_id = Uuid::parse_str(&unit.id).unwrap_or_else(|_| Uuid::new_v4());
        let submission_id = Uuid::parse_str(&unit.submission_id).unwrap_or_else(|_| Uuid::new_v4());

        // Using sqlx::query! macro for compile-time verification
        sqlx::query!(
            r#"
            INSERT INTO submission_units
            (id, submission_id, sequence_number, code, code_system, status_code)
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
            unit_id,
            submission_id,
            unit.sequence_number as i32, // Rust u32 -> Postgres INTEGER
            unit.code,
            unit.code_system,
            "active" // Default status for new units
        )
        .execute(&mut *tx)
        .await?;

        // ---------------------------------------------------------
        // LEVEL 2: Insert Documents (The Physical Files)
        // ---------------------------------------------------------
        for doc in &unit.documents {
            let doc_id = Uuid::parse_str(&doc.id).unwrap_or_else(|_| Uuid::new_v4());

            sqlx::query!(
                r#"
                INSERT INTO documents
                (id, submission_unit_id, xlink_href, checksum, checksum_algorithm, title)
                VALUES ($1, $2, $3, $4, $5, $6)
                "#,
                doc_id,
                unit_id,
                doc.href,
                doc.checksum,
                doc.checksum_algorithm,
                doc.title
            )
            .execute(&mut *tx)
            .await?;
        }

        // ---------------------------------------------------------
        // LEVEL 3: Insert Keyword Definitions (The Vocabulary)
        // ---------------------------------------------------------
        if let Some(definitions) = &unit.keyword_definitions {
            for def in definitions {
                // Note: KeywordDefinition struct needs to be flattened a bit for SQL
                // Assuming we extract the inner value for the loop
                let val = &def.value.item;

                sqlx::query!(
                    r#"
                    INSERT INTO keyword_definitions
                    (submission_unit_id, code, code_system, display_name)
                    VALUES ($1, $2, $3, $4)
                    "#,
                    unit_id,
                    def.code,
                    "urn:oid:2.16.840.1.113883.3.989.2.1.1.1", // Standard eCTD OID or from struct
                    val.display_name
                )
                .execute(&mut *tx)
                .await?;
            }
        }

        // ---------------------------------------------------------
        // LEVEL 4: Insert Context of Use (The Graph)
        // ---------------------------------------------------------
        for cou in &unit.context_of_use {
            let cou_id = Uuid::parse_str(&cou.id).unwrap_or_else(|_| Uuid::new_v4());

            // Handle the optional document reference
            // If this CoU points to a doc, get that UUID.
            let doc_ref_id = cou.document_reference.as_ref()
                .map(|d| Uuid::parse_str(&d.id).unwrap_or(Uuid::nil()));

            sqlx::query!(
                r#"
                INSERT INTO contexts_of_use
                (id, submission_unit_id, code, code_system, status_code, priority_number, document_reference_id)
                VALUES ($1, $2, $3, $4, $5, $6, $7)
                "#,
                cou_id,
                unit_id,
                cou.code,
                "urn:oid:2.16.840.1.113883.3.989.2.1.1.1", // Standard eCTD OID or from struct
                cou.status_code,
                cou.priority_number as i32,
                doc_ref_id
            )
            .execute(&mut *tx)
            .await?;
        }

        // 5. COMMIT THE TRANSACTION
        // Everything is written to disk at this exact microsecond.
        tx.commit().await?;

        Ok(unit_id)
    }
}
