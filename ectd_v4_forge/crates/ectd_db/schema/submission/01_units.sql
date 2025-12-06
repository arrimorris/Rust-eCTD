CREATE TABLE IF NOT EXISTS submission_units (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    submission_id UUID NOT NULL,
    sequence_number INTEGER NOT NULL,
    code VARCHAR(64) NOT NULL,
    code_system VARCHAR(128) NOT NULL,
    status_code VARCHAR(16) NOT NULL CHECK (status_code IN ('active', 'suspended')),
    created_at TIMESTAMPTZ DEFAULT NOW(),

    -- Metadata (from Phase 1 upgrade)
    application_id_uuid UUID NOT NULL DEFAULT gen_random_uuid(),
    application_code VARCHAR(64) NOT NULL DEFAULT 'nda',
    application_number VARCHAR(64) NOT NULL DEFAULT '000000',
    applicant_name VARCHAR(255) NOT NULL DEFAULT 'Unknown',
    submission_code VARCHAR(64) NOT NULL DEFAULT 'seq-0001'
);
