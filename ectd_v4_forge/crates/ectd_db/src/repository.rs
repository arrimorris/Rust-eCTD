use sqlx::{PgPool, FromRow};
use uuid::Uuid;
use ectd_core::models::submission_unit::SubmissionUnit;
// Import other models for cleaner casting
use ectd_core::models::{
    context_of_use::{ContextOfUse, PriorityNumber, DocumentReference, DocumentIdRef},
    document::{Document, DocumentTitle, DocumentText, DocumentReferencePath},
    keyword_definition::{KeywordDefinition, KeywordDefinitionValue, KeywordDefinitionItem, DisplayName},
};

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
        let mut tx = self.pool.begin().await?;

        // ---------------------------------------------------------
        // LEVEL 1: Insert the Submission Unit Container (Updated)
        // ---------------------------------------------------------
        let unit_id = Uuid::parse_str(&unit.id).unwrap_or_else(|_| Uuid::new_v4());
        let submission_id = Uuid::parse_str(&unit.submission.id).unwrap_or_else(|_| Uuid::new_v4());
        let sequence_number = unit.submission.sequence_number.value;

        // Extract new metadata fields
        let app_uuid = Uuid::parse_str(&unit.application.id).unwrap_or_else(|_| Uuid::new_v4());
        let app_code = &unit.application.code;
        let app_num = &unit.application.application_number.code;
        let applicant_name = &unit.applicant.sponsoring_organization.name;
        let sub_code = &unit.submission.code;

        sqlx::query!(
            r#"
            INSERT INTO submission_units
            (id, submission_id, sequence_number, code, code_system, status_code,
             application_id_uuid, application_code, application_number, applicant_name, submission_code)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            "#,
            unit_id,
            submission_id,
            sequence_number as i32,
            unit.code,
            unit.code_system,
            unit.status_code,
            app_uuid,
            app_code,
            app_num,
            applicant_name,
            sub_code
        )
        .execute(&mut *tx)
        .await?;

        // ---------------------------------------------------------
        // LEVEL 2: Insert Documents (Unchanged)
        // ---------------------------------------------------------
        for doc in &unit.documents {
            let doc_id = Uuid::parse_str(&doc.id).unwrap_or_else(|_| Uuid::new_v4());
            let href = &doc.text.reference.value;
            let checksum = &doc.text.checksum;
            let alg = &doc.text.checksum_algorithm;
            let title = &doc.title.value;

            sqlx::query!(
                r#"
                INSERT INTO documents
                (id, submission_unit_id, xlink_href, checksum, checksum_algorithm, title)
                VALUES ($1, $2, $3, $4, $5, $6)
                "#,
                doc_id,
                unit_id,
                href,
                checksum,
                alg,
                title
            )
            .execute(&mut *tx)
            .await?;
        }

        // ---------------------------------------------------------
        // LEVEL 3: Insert Keyword Definitions (Unchanged)
        // ---------------------------------------------------------
        if let Some(definitions) = &unit.keyword_definitions {
            for def in definitions {
                let val = &def.value.item;
                sqlx::query!(
                    r#"
                    INSERT INTO keyword_definitions
                    (submission_unit_id, code, code_system, display_name)
                    VALUES ($1, $2, $3, $4)
                    "#,
                    unit_id,
                    def.code,
                    "urn:oid:2.16.840.1.113883.3.989.2.1.1.1",
                    val.display_name.value
                )
                .execute(&mut *tx)
                .await?;
            }
        }

        // ---------------------------------------------------------
        // LEVEL 4: Insert Context of Use (Unchanged)
        // ---------------------------------------------------------
        for cou in &unit.context_of_use {
            let cou_id = Uuid::parse_str(&cou.id).unwrap_or_else(|_| Uuid::new_v4());
            let doc_ref_id = cou.document_reference.as_ref()
                .map(|d| Uuid::parse_str(&d.id.root).unwrap_or(Uuid::nil()));

            sqlx::query!(
                r#"
                INSERT INTO contexts_of_use
                (id, submission_unit_id, code, code_system, status_code, priority_number, document_reference_id)
                VALUES ($1, $2, $3, $4, $5, $6, $7)
                "#,
                cou_id,
                unit_id,
                cou.code,
                cou.code_system,
                cou.status_code,
                cou.priority_number.value as i32,
                doc_ref_id
            )
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;
        Ok(unit_id)
    }

    /// Reconstructs a full SubmissionUnit from the relational database
    pub async fn get_submission(&self, id: Uuid) -> Result<SubmissionUnit, sqlx::Error> {
        // 1. Fetch Root (Updated Select)
        let unit_rec = sqlx::query!(
            r#"
            SELECT id, submission_id, sequence_number, code, code_system, status_code, created_at,
                   application_id_uuid, application_code, application_number, applicant_name, submission_code
            FROM submission_units
            WHERE id = $1
            "#,
            id
        ).fetch_one(&self.pool).await?;

        // 2. Fetch Documents (Using centralized map)
        let documents: Vec<Document> = sqlx::query_as!(DocumentRow,
            r#"SELECT id, xlink_href, checksum, checksum_algorithm, title, media_type FROM documents WHERE submission_unit_id = $1"#,
            id
        )
        .fetch_all(&self.pool)
        .await?
        .into_iter()
        .map(Into::into)
        .collect();

        // 3. Fetch Contexts
        let context_of_use: Vec<ContextOfUse> = sqlx::query_as!(ContextRow,
            r#"SELECT id, code, code_system, status_code, priority_number, document_reference_id FROM contexts_of_use WHERE submission_unit_id = $1"#,
            id
        )
        .fetch_all(&self.pool)
        .await?
        .into_iter()
        .map(Into::into)
        .collect();

        // 4. Fetch Keywords
        let keywords_raw = sqlx::query_as!(KeywordRow,
            r#"SELECT code, code_system, display_name FROM keyword_definitions WHERE submission_unit_id = $1"#,
            id
        )
        .fetch_all(&self.pool)
        .await?;

        let keyword_definitions = if keywords_raw.is_empty() {
            None
        } else {
            Some(keywords_raw.into_iter().map(Into::into).collect())
        };

        // 5. Assemble (Updated with Real Data)
        Ok(SubmissionUnit {
            xmlns: "urn:hl7-org:v3".to_string(),
            xmlns_xsi: None,
            schema_location: None,
            id: unit_rec.id.to_string(),
            code: unit_rec.code,
            code_system: unit_rec.code_system,
            status_code: unit_rec.status_code,

            // Real Submission Metadata
            submission: ectd_core::models::submission_unit::Submission {
                id: unit_rec.submission_id.to_string(),
                code: unit_rec.submission_code,
                code_system: "urn:oid:2.16.840.1.113883.3.989.2.2.1".to_string(),
                sequence_number: ectd_core::models::submission_unit::SequenceNumber {
                    value: unit_rec.sequence_number as u32,
                },
            },

            // Real Application Metadata
            application: ectd_core::models::submission_unit::Application {
                id: unit_rec.application_id_uuid.to_string(),
                code: unit_rec.application_code,
                code_system: "urn:oid:2.16.840.1.113883.3.989.2.2.1".to_string(),
                application_number: ectd_core::models::submission_unit::ApplicationNumber {
                    code: unit_rec.application_number,
                    code_system: "urn:oid:2.16.840.1.113883.3.989.2.2.1".to_string(),
                },
            },

            // Real Applicant Metadata
            applicant: ectd_core::models::submission_unit::Applicant {
                sponsoring_organization: ectd_core::models::submission_unit::SponsoringOrganization {
                    name: unit_rec.applicant_name,
                },
            },

            context_of_use,
            documents,
            keyword_definitions,
        })
    }
}

// =================================================================
// THE MAP (Internal Structs & Converters)
// =================================================================

#[derive(FromRow)]
struct DocumentRow {
    id: Uuid,
    xlink_href: String,
    checksum: String,
    checksum_algorithm: Option<String>,
    title: String,
    media_type: Option<String>,
}

impl Into<Document> for DocumentRow {
    fn into(self) -> Document {
        Document {
            id: self.id.to_string(),
            title: DocumentTitle { value: self.title },
            text: DocumentText {
                reference: DocumentReferencePath { value: self.xlink_href },
                checksum: self.checksum,
                checksum_algorithm: self.checksum_algorithm.unwrap_or_else(|| "SHA256".to_string()),
                media_type: self.media_type.unwrap_or_else(|| "application/pdf".to_string()),
            },
        }
    }
}

#[derive(FromRow)]
struct ContextRow {
    id: Uuid,
    code: String,
    code_system: String,
    status_code: String,
    priority_number: i32,
    document_reference_id: Option<Uuid>,
}

impl Into<ContextOfUse> for ContextRow {
    fn into(self) -> ContextOfUse {
        ContextOfUse {
            id: self.id.to_string(),
            code: self.code,
            code_system: self.code_system,
            status_code: self.status_code,
            priority_number: PriorityNumber { value: self.priority_number as u32 },
            document_reference: self.document_reference_id.map(|id| DocumentReference {
                id: DocumentIdRef { root: id.to_string() }
            }),
            related_context_of_use: None,
            keywords: vec![],
        }
    }
}

#[derive(FromRow)]
struct KeywordRow {
    code: String,
    code_system: String,
    display_name: String,
}

impl Into<KeywordDefinition> for KeywordRow {
    fn into(self) -> KeywordDefinition {
        KeywordDefinition {
            code: self.code.clone(),
            code_system: self.code_system,
            value: KeywordDefinitionValue {
                item: KeywordDefinitionItem {
                    code: self.code,
                    display_name: DisplayName { value: self.display_name },
                },
            },
        }
    }
}
