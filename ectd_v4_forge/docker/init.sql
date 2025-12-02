-- 1. Enable UUIDv7 extension (Native in PG18, or via pg_uuidv7 in older versions)
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- ========================================================
-- LEVEL 1: The Container (Submission Unit)
-- Reference: PDF Section 4.2.2 [cite: 13]
-- ========================================================
CREATE TABLE submission_units (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(), -- eCTD4-004: Must be UUID

    -- The "Logical" ID that persists across sequences for the same submission
    submission_id UUID NOT NULL, -- eCTD4-033

    -- e.g., "0001", "0002". Must be unique per application
    sequence_number INTEGER NOT NULL CHECK (sequence_number > 0 AND sequence_number < 999999), -- eCTD4-013

    -- Metadata
    code VARCHAR(64) NOT NULL, -- eCTD4-006
    code_system VARCHAR(128) NOT NULL, -- eCTD4-008
    status_code VARCHAR(16) NOT NULL CHECK (status_code IN ('active', 'suspended')), -- eCTD4-010

    -- Audit fields
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- ========================================================
-- LEVEL 2: The Vocabulary (Keyword Definitions)
-- Reference: PDF Section 4.2.14 [cite: 181]
-- ========================================================
CREATE TABLE keyword_definitions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    submission_unit_id UUID REFERENCES submission_units(id) ON DELETE CASCADE,

    code VARCHAR(64) NOT NULL, -- eCTD4-052
    code_system VARCHAR(128) NOT NULL, -- eCTD4-083
    display_name VARCHAR(512) NOT NULL, -- eCTD4-058

    UNIQUE(submission_unit_id, code) -- Prevent duplicates in same unit
);

-- ========================================================
-- LEVEL 3: The Payload (Documents)
-- Reference: PDF Section 4.2.13 [cite: 164]
-- ========================================================
CREATE TABLE documents (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(), -- eCTD4-045
    submission_unit_id UUID REFERENCES submission_units(id) ON DELETE CASCADE,

    -- The physical file reference
    xlink_href TEXT NOT NULL, -- eCTD4-050
    media_type VARCHAR(64) DEFAULT 'application/pdf',

    -- Integrity [cite: 172]
    checksum VARCHAR(64) NOT NULL, -- SHA-256 hash
    checksum_algorithm VARCHAR(16) DEFAULT 'SHA-256',

    title VARCHAR(512) NOT NULL, -- eCTD4-047

    -- Constraints
    CONSTRAINT unique_doc_id_per_unit UNIQUE (id, submission_unit_id)
);

-- ========================================================
-- LEVEL 4: The Graph (Context of Use)
-- Reference: PDF Section 4.2.5 [cite: 85]
-- ========================================================
CREATE TABLE contexts_of_use (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(), -- eCTD4-021
    submission_unit_id UUID REFERENCES submission_units(id) ON DELETE CASCADE,

    -- What is this doc? (Links to Keyword Definitions or Standard Codes)
    code VARCHAR(64) NOT NULL, -- eCTD4-075
    code_system VARCHAR(128) NOT NULL,

    -- Lifecycle Management
    status_code VARCHAR(16) NOT NULL CHECK (status_code IN ('active', 'suspended')), -- eCTD4-023

    -- Ordering
    priority_number INTEGER NOT NULL CHECK (priority_number > 0), -- eCTD4-017

    -- The Link to the Document (Optional because 'suspended' CoUs might not have docs)
    document_reference_id UUID REFERENCES documents(id), -- eCTD4-027

    -- Relationship to OTHER Contexts (e.g., Replacing an old doc)
    replaces_context_id UUID -- eCTD4-026: Points to a previous CoU ID
);

-- ========================================================
-- LEVEL 5: Many-to-Many Keywords (The "Tags")
-- Reference: PDF Section 4.2.8 [cite: 110]
-- ========================================================
CREATE TABLE context_keywords (
    context_id UUID REFERENCES contexts_of_use(id) ON DELETE CASCADE,

    code VARCHAR(64) NOT NULL, -- eCTD4-029
    code_system VARCHAR(128) NOT NULL, -- eCTD4-030

    PRIMARY KEY (context_id, code)
);
