CREATE TABLE documents (
    id VARCHAR(255) PRIMARY KEY, -- UUID string from XML usually, or UUIDv7
    submission_unit_id UUID NOT NULL REFERENCES submission_units(id) ON DELETE CASCADE,

    xlink_href VARCHAR(1024) NOT NULL,
    checksum VARCHAR(255) NOT NULL,
    checksum_algorithm VARCHAR(64) NOT NULL, -- "SHA256"

    title VARCHAR(1024),
    media_type VARCHAR(255) NOT NULL DEFAULT 'application/pdf',

    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_docs_unit ON documents(submission_unit_id);
