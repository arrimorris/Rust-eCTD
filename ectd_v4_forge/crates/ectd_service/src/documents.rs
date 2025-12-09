use crate::EctdService;
use anyhow::{Context, Result};
use uuid::Uuid;
use tokio::fs::File;
use tokio::io::AsyncReadExt;
use sha2::{Sha256, Digest};
use aws_sdk_s3::primitives::ByteStream;

use ectd_core::models::{
    document::{Document, DocumentTitle, DocumentText, DocumentReferencePath},
    context_of_use::{ContextOfUse, PriorityNumber, DocumentReference, DocumentIdRef},
    submission_unit::{SubmissionUnit, Submission, Application, Applicant, SequenceNumber, ApplicationNumber, SponsoringOrganization},
};
// Import the new helper
use ectd_core::resolve_folder_path;
use ectd_core::validation::{ValidationEngine, rules_pdf::RuleEctd4_533};
use ectd_db::repository::SubmissionRepository;

#[derive(Debug)]
pub struct AddDocumentParams {
    pub submission_id: Uuid,
    pub file_path: std::path::PathBuf,
    pub context_code: String,
    pub title: String,
    pub priority: u32,
}

impl EctdService {
    pub async fn attach_document(&self, params: AddDocumentParams) -> Result<Uuid> {
        // 0. SELF-HEALING: Ensure Vault is ready
        self.ensure_bucket().await
            .context("Failed to initialize storage backend")?;

        // 1. Checksum (Streaming from disk)
        let mut file = File::open(&params.file_path).await
            .context(format!("Failed to open file: {:?}", params.file_path))?;

        let mut hasher = Sha256::new();
        let mut buffer = [0; 8192]; // 8KB chunks
        loop {
            let n = file.read(&mut buffer).await?;
            if n == 0 { break; }
            hasher.update(&buffer[..n]);
        }
        let hash = hex::encode(hasher.finalize());

        // 1.5 VALIDATION (The Shield)
        // Check PDF integrity before uploading.
        // We assume "application/pdf" for now, but in reality we should check extension.
        if let Some(ext) = params.file_path.extension() {
            if ext.to_string_lossy().to_lowercase() == "pdf" {
                // Construct a minimal dummy unit to satisfy the Validator signature
                let validation_doc = Document {
                    id: "temp-validation-id".to_string(),
                    title: DocumentTitle { value: params.title.clone() },
                    text: DocumentText {
                        // Crucial: Use LOCAL path for validation so lopdf can find it
                        reference: DocumentReferencePath { value: params.file_path.to_string_lossy().to_string() },
                        checksum: hash.clone(),
                        checksum_algorithm: "SHA256".to_string(),
                        media_type: "application/pdf".to_string(),
                    },
                };

                let dummy_unit = SubmissionUnit {
                    id: Uuid::new_v4().to_string(),
                    code: "validation-wrapper".to_string(),
                    code_system: "urn:oid:2.16.840.1.113883.3.989.2.2.1".to_string(),
                    status_code: "active".to_string(),
                    xmlns: "urn:hl7-org:v3".to_string(),
                    xmlns_xsi: None,
                    schema_location: None,
                    submission: Submission {
                        id: Uuid::new_v4().to_string(),
                        code: "seq-0000".to_string(),
                        code_system: "urn:oid:2.16.840.1.113883.3.989.2.2.1".to_string(),
                        sequence_number: SequenceNumber { value: 0 },
                    },
                    application: Application {
                        id: Uuid::new_v4().to_string(),
                        code: "nda".to_string(),
                        code_system: "urn:oid:2.16.840.1.113883.3.989.2.2.1".to_string(),
                        application_number: ApplicationNumber {
                            code: "000000".to_string(),
                            code_system: "urn:oid:2.16.840.1.113883.3.989.2.2.1".to_string(),
                        }
                    },
                    applicant: Applicant {
                        sponsoring_organization: SponsoringOrganization { name: "Validation".to_string() }
                    },
                    context_of_use: vec![],
                    keyword_definitions: None,
                    documents: vec![validation_doc],
                };

                let engine = ValidationEngine::new()
                    .add_rule(RuleEctd4_533);

                let errors = engine.run(&dummy_unit);

                // Block on High Errors (Severity "High Error")
                for err in errors {
                    if err.severity.contains("High Error") {
                         anyhow::bail!("PDF Validation Failed: {} (Code: {})", err.message, err.code);
                    }
                    // We could log warnings here
                }
            }
        }

        // 2. Identities
        let doc_id = Uuid::new_v4();
        let cou_id = Uuid::new_v4();

        // 3. Upload (Streaming again)
        let body = ByteStream::from_path(&params.file_path).await?;

        self.s3.put_object()
            .bucket(&self.bucket)
            .key(doc_id.to_string())
            .body(body)
            .content_type("application/pdf")
            .send()
            .await
            .context("S3 Upload Failed")?;

        // 4. Construct
        let filename = params.file_path.file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        // Use the resolver to determine the correct eCTD folder
        let folder = resolve_folder_path(&params.context_code);
        let ref_path = format!("{}/{}", folder, filename);

        let doc = Document {
            id: doc_id.to_string(),
            title: DocumentTitle { value: params.title },
            text: DocumentText {
                reference: DocumentReferencePath { value: ref_path },
                checksum: hash,
                checksum_algorithm: "SHA256".to_string(),
                media_type: "application/pdf".to_string(),
            },
        };

        let cou = ContextOfUse {
            id: cou_id.to_string(),
            code: params.context_code,
            code_system: "urn:oid:2.16.840.1.113883.3.989.2.2.1".to_string(),
            status_code: "active".to_string(),
            priority_number: PriorityNumber { value: params.priority },
            document_reference: Some(DocumentReference {
                id: DocumentIdRef { root: doc_id.to_string() }
            }),
            related_context_of_use: None,
            keywords: vec![],
        };

        // 5. Persist
        let repo = SubmissionRepository::new(self.pool.clone());
        repo.add_document_to_submission(params.submission_id, &doc, &cou).await?;

        Ok(doc_id)
    }
}
