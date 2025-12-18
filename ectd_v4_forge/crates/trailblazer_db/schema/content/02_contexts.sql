CREATE TABLE contexts_of_use (
    id VARCHAR(255) PRIMARY KEY,
    submission_unit_id UUID NOT NULL REFERENCES submission_units(id) ON DELETE CASCADE,

    code VARCHAR(64) NOT NULL, -- e.g. "clinical-dataset"
    code_system VARCHAR(255) NOT NULL,
    status_code VARCHAR(64) NOT NULL,

    priority_number INTEGER,

    -- Link to Document
    document_reference_id VARCHAR(255) REFERENCES documents(id),

    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_cou_unit ON contexts_of_use(submission_unit_id);
