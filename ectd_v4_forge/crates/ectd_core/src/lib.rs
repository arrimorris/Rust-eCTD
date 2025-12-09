pub mod models;
pub mod validation;

use validation::{ValidationEngine, rules, rules_pdf};

pub fn get_standard_validator() -> ValidationEngine {
    ValidationEngine::new()
        .add_rule(rules::RuleEctd4_004)
        .add_rule(rules::RuleEctd4_006)
        .add_rule(rules::RuleEctd4_013)
        .add_rule(rules::RuleEctd4_048)
        .add_rule(rules_pdf::RuleEctd4_533)
}

/// Maps an eCTD v4.0 Context of Use Code to the standard folder path.
/// While eCTD v4.0 allows a flat structure, organizing by module is best practice.
pub fn resolve_folder_path(context_code: &str) -> &'static str {
    match context_code {
        // Module 1: Administrative
        "cover-letter" => "m1/us",
        "product-labeling" => "m1/us",
        "form-356h" => "m1/us",
        "form-1571" => "m1/us",

        // Module 2: Summaries
        "quality-overall-summary" => "m2/23-qos",
        "clinical-overview" => "m2/25-clin-over",
        "nonclinical-overview" => "m2/24-nonclin-over",

        // Module 3: Quality
        "drug-substance" => "m3/32-body-data/32s-drug-sub",
        "drug-product" => "m3/32-body-data/32p-drug-prod",
        "regional-information" => "m3/32-body-data/32r-reg-info",

        // Module 4: Nonclinical
        "nonclinical-study-report" => "m4/42-study-rep",
        "toxicology" => "m4/42-study-rep/423-tox",
        "pharmacology" => "m4/42-study-rep/421-pharm",

        // Module 5: Clinical
        "clinical-study-report" => "m5/53-clin-study-rep",
        "clinical-dataset" => "m5/datasets",
        "analysis-dataset" => "m5/datasets",
        "data-definition" => "m5/datasets", // define.xml
        "case-report-forms" => "m5/53-clin-study-rep/537-crf-ipl",

        // Fallback / Generic
        _ => "m1/us",
    }
}

#[cfg(test)]
mod tests {
    use crate::models::submission_unit::SubmissionUnit;
    use quick_xml::de::from_str;

    // The raw XML from your 'sample_submission.xml' file
    const SAMPLE_XML: &str = r#"
    <submissionUnit xmlns="urn:hl7-org:v3" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
        xsi:schemaLocation="urn:hl7-org:v3 ../../schema/rps.xsd"
        id="12345678-1234-1234-1234-123456789012"
        code="original-application" codeSystem="2.16.840.1.113883.3.989.2.2.1"
        statusCode="active">

        <submission id="aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa" code="seq-0001" codeSystem="2.16.840.1.113883.3.989.2.2.1">
            <sequenceNumber value="0001"/>
        </submission>

        <application id="bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb" code="nda" codeSystem="2.16.840.1.113883.3.989.2.2.1">
            <code code="123456" codeSystem="2.16.840.1.113883.3.989.2.2.1"/>
        </application>

        <applicant>
            <sponsoringOrganization>
                <name>Acme Pharmaceuticals</name>
            </sponsoringOrganization>
        </applicant>

        <contextOfUse id="cccccccc-cccc-cccc-cccc-cccccccccccc" code="cover-letter" codeSystem="2.16.840.1.113883.3.989.2.2.1" statusCode="active">
            <priorityNumber value="1"/>
            <documentReference>
                <id root="dddddddd-dddd-dddd-dddd-dddddddddddd"/>
            </documentReference>
        </contextOfUse>

        <document id="dddddddd-dddd-dddd-dddd-dddddddddddd">
            <title value="Cover Letter"/>
            <text integrityCheck="e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855" integrityCheckAlgorithm="SHA256" mediaType="application/pdf">
                <reference value="m1/us/cover.pdf"/>
            </text>
        </document>

        <keywordDefinition code="my-term" codeSystem="2.16.840.1.113883.3.989.2.2.1">
            <value>
                <item code="my-term" displayName="My Custom Term">
                    <displayName value="My Custom Term"/>
                </item>
            </value>
        </keywordDefinition>
    </submissionUnit>
    "#;

    #[test]
    fn test_parse_sample_submission() {
        // 1. Kinetic Energy: Attempt to parse the XML string into our Rust Struct
        let result: Result<SubmissionUnit, _> = from_str(SAMPLE_XML);

        // Fail fast if parsing explodes (e.g. malformed XML or missing fields)
        assert!(result.is_ok(), "Failed to parse XML: {:?}", result.err());
        let unit = result.unwrap();

        // 2. Structural Integrity Checks
        // Verify the Root Attributes
        assert_eq!(unit.id, "12345678-1234-1234-1234-123456789012");
        assert_eq!(unit.code, "original-application");
        assert_eq!(unit.status_code, "active");

        // Verify Nested Metadata (Submission)
        assert_eq!(unit.submission.code, "seq-0001");
        assert_eq!(unit.submission.sequence_number.value, 1);

        // Verify Application Info
        assert_eq!(unit.application.code, "nda");
        assert_eq!(unit.application.application_number.code, "123456");

        // Verify Applicant
        assert_eq!(unit.applicant.sponsoring_organization.name, "Acme Pharmaceuticals");

        // 3. Graph Logic Checks
        // Context of Use (The Edge)
        assert_eq!(unit.context_of_use.len(), 1);
        let cou = &unit.context_of_use[0];
        assert_eq!(cou.code, "cover-letter");
        assert_eq!(cou.priority_number.value, 1);

        // Verify the Document Reference inside the CoU
        assert!(cou.document_reference.is_some());
        let doc_ref = cou.document_reference.as_ref().unwrap();
        assert_eq!(doc_ref.id.root, "dddddddd-dddd-dddd-dddd-dddddddddddd");

        // Document (The Node)
        assert_eq!(unit.documents.len(), 1);
        let doc = &unit.documents[0];
        assert_eq!(doc.id, "dddddddd-dddd-dddd-dddd-dddddddddddd");
        assert_eq!(doc.title.value, "Cover Letter");

        // Verify Physical File Path
        assert_eq!(doc.text.reference.value, "m1/us/cover.pdf");
        assert_eq!(doc.text.checksum_algorithm, "SHA256");

        // 4. Vocabulary Check
        // Keyword Definition
        assert!(unit.keyword_definitions.is_some());
        let keywords = unit.keyword_definitions.as_ref().unwrap();
        assert_eq!(keywords.len(), 1);
        let kw = &keywords[0];
        assert_eq!(kw.code, "my-term");
        assert_eq!(kw.value.item.display_name.value, "My Custom Term");
    }
}
pub mod sdtm;
