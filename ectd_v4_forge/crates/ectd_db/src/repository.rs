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

        // Extract from nested structs (HL7 RPS structure)
        let submission_id = Uuid::parse_str(&unit.submission.id).unwrap_or_else(|_| Uuid::new_v4());
        let sequence_number = unit.submission.sequence_number.value;

        // Using sqlx::query! macro for compile-time verification
        sqlx::query!(
            r#"
            INSERT INTO submission_units
            (id, submission_id, sequence_number, code, code_system, status_code)
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
            unit_id,
            submission_id,
            sequence_number as i32, // Rust u32 -> Postgres INTEGER
            unit.code,
            unit.code_system,
            unit.status_code
        )
        .execute(&mut *tx)
        .await?;

        // ---------------------------------------------------------
        // LEVEL 2: Insert Documents (The Physical Files)
        // ---------------------------------------------------------
        for doc in &unit.documents {
            let doc_id = Uuid::parse_str(&doc.id).unwrap_or_else(|_| Uuid::new_v4());

            // Map nested HL7 fields to flat SQL
            // doc.text.reference.value -> xlink_href
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
                    val.display_name.value
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
            // In v4 XML, it's doc_ref -> id -> @root
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

        // 5. COMMIT THE TRANSACTION
        // Everything is written to disk at this exact microsecond.
        tx.commit().await?;

        Ok(unit_id)
    }

    /// Reconstructs a full SubmissionUnit from the relational database
    pub async fn get_submission(&self, id: Uuid) -> Result<SubmissionUnit, sqlx::Error> {
        // 1. Fetch the Root (Submission Unit)
        let unit_rec = sqlx::query!(
            r#"
            SELECT id, submission_id, sequence_number, code, code_system, status_code, created_at
            FROM submission_units
            WHERE id = $1
            "#,
            id
        )
        .fetch_one(&self.pool)
        .await?;

        // 2. Fetch All Documents for this Unit
        let docs_recs = sqlx::query!(
            r#"
            SELECT id, xlink_href, checksum, checksum_algorithm, title, media_type
            FROM documents
            WHERE submission_unit_id = $1
            "#,
            id
        )
        .fetch_all(&self.pool)
        .await?;

        // Map DB Documents to Rust Structs
        let documents: Vec<ectd_core::models::document::Document> = docs_recs
            .into_iter()
            .map(|r| ectd_core::models::document::Document {
                id: r.id.to_string(),
                title: ectd_core::models::document::DocumentTitle { value: r.title },
                text: ectd_core::models::document::DocumentText {
                    reference: ectd_core::models::document::DocumentReferencePath { value: r.xlink_href },
                    checksum: r.checksum,
                    checksum_algorithm: r.checksum_algorithm.unwrap_or_else(|| "SHA256".to_string()),
                    media_type: r.media_type.unwrap_or_else(|| "application/pdf".to_string()),
                },
            })
            .collect();

        // 3. Fetch All Contexts of Use
        let cou_recs = sqlx::query!(
            r#"
            SELECT id, code, code_system, status_code, priority_number, document_reference_id
            FROM contexts_of_use
            WHERE submission_unit_id = $1
            "#,
            id
        )
        .fetch_all(&self.pool)
        .await?;

        // Map DB Contexts to Rust Structs
        let context_of_use: Vec<ectd_core::models::context_of_use::ContextOfUse> = cou_recs
            .into_iter()
            .map(|r| ectd_core::models::context_of_use::ContextOfUse {
                id: r.id.to_string(),
                code: r.code,
                code_system: r.code_system,
                status_code: r.status_code,
                priority_number: ectd_core::models::context_of_use::PriorityNumber {
                    value: r.priority_number as u32,
                },
                document_reference: r.document_reference_id.map(|doc_id| {
                    ectd_core::models::context_of_use::DocumentReference {
                        id: ectd_core::models::context_of_use::DocumentIdRef {
                            root: doc_id.to_string(),
                        },
                    }
                }),
                related_context_of_use: None, // Not yet implemented in DB schema
                keywords: vec![],             // To be implemented with join table
            })
            .collect();

        // 4. Fetch Keyword Definitions
        let kw_recs = sqlx::query!(
            r#"
            SELECT code, code_system, display_name
            FROM keyword_definitions
            WHERE submission_unit_id = $1
            "#,
            id
        )
        .fetch_all(&self.pool)
        .await?;

        let keyword_definitions: Vec<ectd_core::models::keyword_definition::KeywordDefinition> = kw_recs
            .into_iter()
            .map(|r| ectd_core::models::keyword_definition::KeywordDefinition {
                code: r.code.clone(),
                code_system: r.code_system,
                value: ectd_core::models::keyword_definition::KeywordDefinitionValue {
                    item: ectd_core::models::keyword_definition::KeywordDefinitionItem {
                        code: r.code,
                        display_name: ectd_core::models::keyword_definition::DisplayName {
                            value: r.display_name,
                        },
                    },
                },
            })
            .collect();

        // Wrap keyword_definitions in Option
        let keyword_definitions = if keyword_definitions.is_empty() {
            None
        } else {
            Some(keyword_definitions)
        };

        // 5. Assemble the Titan
        // Note: We fill missing DB fields with placeholders to ensure valid JSON output
        Ok(SubmissionUnit {
            xmlns: "urn:hl7-org:v3".to_string(),
            xmlns_xsi: None,
            schema_location: None,
            id: unit_rec.id.to_string(),
            code: unit_rec.code,
            code_system: unit_rec.code_system,
            status_code: unit_rec.status_code,

            // Reconstruct Submission Block
            submission: ectd_core::models::submission_unit::Submission {
                id: unit_rec.submission_id.to_string(),
                code: "seq-0001".to_string(), // Placeholder until we store submission code
                code_system: "urn:oid:submission-code-system".to_string(),
                sequence_number: ectd_core::models::submission_unit::SequenceNumber {
                    value: unit_rec.sequence_number as u32,
                },
            },

            // Placeholders for Application/Applicant (Not yet in DB)
            application: ectd_core::models::submission_unit::Application {
                id: Uuid::nil().to_string(),
                code: "placeholder".to_string(),
                code_system: "urn:oid:placeholder".to_string(),
                application_number: ectd_core::models::submission_unit::ApplicationNumber {
                    code: "000000".to_string(),
                    code_system: "urn:oid:placeholder".to_string(),
                },
            },
            applicant: ectd_core::models::submission_unit::Applicant {
                sponsoring_organization: ectd_core::models::submission_unit::SponsoringOrganization {
                    name: "Stored in DB (Placeholder)".to_string(),
                },
            },

            context_of_use,
            documents,
            keyword_definitions,
        })
    }
}
