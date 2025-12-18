-- =====================================================================
-- TRAILBLAZER FORGE: The Unified Schema
-- =====================================================================

-- [Phase 0] Foundation
-- @include foundation/00_extensions.sql
-- @include foundation/01_tenancy.sql

-- [Phase 1] The CTMS (Live Operations)
-- @include core/01_departments.sql
-- @include inventory/01_categories.sql
-- @include inventory/02_items.sql

-- [Phase 2] The eCTD v4.0 System
-- @include submission/01_units.sql
-- @include content/01_documents.sql
-- @include content/02_contexts.sql
-- @include content/03_keywords.sql

-- [Phase 3] The Bridge
-- @include bridge/01_submission_mappings.sql
