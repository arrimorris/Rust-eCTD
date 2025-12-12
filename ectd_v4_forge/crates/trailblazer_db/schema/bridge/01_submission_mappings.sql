CREATE TABLE submission_mappings (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    submission_unit_id UUID NOT NULL REFERENCES submission_units(id),
    source_table VARCHAR(64) NOT NULL,
    source_id UUID NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(submission_unit_id, source_table, source_id)
);
