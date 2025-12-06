CREATE TABLE IF NOT EXISTS contexts_of_use (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    submission_unit_id UUID REFERENCES submission_units(id) ON DELETE CASCADE,
    code VARCHAR(64) NOT NULL,
    code_system VARCHAR(128) NOT NULL,
    status_code VARCHAR(16) NOT NULL CHECK (status_code IN ('active', 'suspended')),
    priority_number INTEGER NOT NULL CHECK (priority_number > 0),
    document_reference_id UUID REFERENCES documents(id),
    replaces_context_id UUID
);
