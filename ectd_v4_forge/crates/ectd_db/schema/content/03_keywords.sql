CREATE TABLE IF NOT EXISTS keyword_definitions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    submission_unit_id UUID REFERENCES submission_units(id) ON DELETE CASCADE,
    code VARCHAR(64) NOT NULL,
    code_system VARCHAR(128) NOT NULL,
    display_name VARCHAR(512) NOT NULL,
    UNIQUE(submission_unit_id, code)
);
