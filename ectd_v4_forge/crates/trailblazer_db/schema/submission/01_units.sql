CREATE TABLE submission_units (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    submission_id UUID NOT NULL,
    sequence_number INTEGER NOT NULL,
    code VARCHAR(64) NOT NULL, -- e.g. "submission-unit"
    code_system VARCHAR(255) NOT NULL,
    status_code VARCHAR(64) NOT NULL, -- e.g. "active"

    -- Metadata
    created_at TIMESTAMPTZ DEFAULT NOW(),

    -- Application / Applicant Info (Denormalized or JSONB could be used, but strict columns preferred)
    application_id_uuid UUID, -- The Application ID
    application_code VARCHAR(64),
    application_number VARCHAR(64),
    applicant_name VARCHAR(255),
    submission_code VARCHAR(64), -- e.g. "seq-0000"

    -- XML Namespace info
    xmlns VARCHAR(255),
    xmlns_xsi VARCHAR(255),
    schema_location VARCHAR(255)
);

CREATE INDEX idx_sub_units_app ON submission_units(application_number);
