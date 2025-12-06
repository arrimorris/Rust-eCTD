CREATE TABLE IF NOT EXISTS documents (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    submission_unit_id UUID REFERENCES submission_units(id) ON DELETE CASCADE,
    xlink_href TEXT NOT NULL,
    media_type VARCHAR(64) DEFAULT 'application/pdf',
    checksum VARCHAR(64) NOT NULL,
    checksum_algorithm VARCHAR(16) DEFAULT 'SHA-256',
    title VARCHAR(512) NOT NULL,
    CONSTRAINT unique_doc_id_per_unit UNIQUE (id, submission_unit_id)
);
