CREATE TABLE keyword_definitions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    submission_unit_id UUID NOT NULL REFERENCES submission_units(id) ON DELETE CASCADE,

    code VARCHAR(64),
    code_system VARCHAR(255),
    display_name VARCHAR(255),

    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_keywords_unit ON keyword_definitions(submission_unit_id);
