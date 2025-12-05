use crate::EctdService;
use anyhow::{Context, Result};
use uuid::Uuid;
use ectd_core::models::submission_unit::{
    SubmissionUnit, Submission, Application, ApplicationNumber, Applicant, SponsoringOrganization, SequenceNumber
};
use ectd_db::repository::SubmissionRepository;

#[derive(Debug)]
pub struct InitSubmissionParams {
    pub app_number: String,
    pub app_type: String, // Added field
    pub applicant_name: String,
    pub sequence_number: u32,
    pub submission_code: String,
}

impl EctdService {
    pub async fn create_submission(&self, params: InitSubmissionParams) -> Result<Uuid> {
        let submission_uuid = Uuid::new_v4();
        let app_uuid = Uuid::new_v4();
        let unit_id = Uuid::new_v4();

        let unit = SubmissionUnit {
            xmlns: "urn:hl7-org:v3".to_string(),
            xmlns_xsi: Some("http://www.w3.org/2001/XMLSchema-instance".to_string()),
            schema_location: Some("urn:hl7-org:v3 ../../util/dtd/v3_0/schema/rps_schema.xsd".to_string()),
            id: unit_id.to_string(),
            code: "submission-unit".to_string(),
            code_system: "urn:oid:2.16.840.1.113883.3.989.2.2.1".to_string(),
            status_code: "active".to_string(),

            submission: Submission {
                id: submission_uuid.to_string(),
                code: params.submission_code,
                code_system: "urn:oid:2.16.840.1.113883.3.989.2.2.1".to_string(),
                sequence_number: SequenceNumber { value: params.sequence_number },
            },

            application: Application {
                id: app_uuid.to_string(),
                code: params.app_type, // Use parameter
                code_system: "urn:oid:2.16.840.1.113883.3.989.2.2.1".to_string(),
                application_number: ApplicationNumber {
                    code: params.app_number,
                    code_system: "urn:oid:2.16.840.1.113883.3.989.2.2.1".to_string(),
                },
            },

            applicant: Applicant {
                sponsoring_organization: SponsoringOrganization {
                    name: params.applicant_name,
                },
            },

            context_of_use: vec![],
            documents: vec![],
            keyword_definitions: None,
        };

        let repo = SubmissionRepository::new(self.pool.clone());
        repo.create_submission(&unit).await
            .context("Failed to persist submission unit")?;

        Ok(unit_id)
    }
}
