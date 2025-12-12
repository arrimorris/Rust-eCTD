#!/bin/bash
set -e

echo "Starting Trailblazer Refactor..."

CDIR="ectd_v4_forge/crates"

# 1. Reset/Clean (optional, assumes starting from ectd_v4_forge state)
# If trailblazer_db exists, we assume partial state and try to clean up or just overwrite.
# For safety, we assume we are renaming existing ectd_* directories.

if [ -d "$CDIR/ectd_db" ]; then
    echo "Renaming ectd_db -> trailblazer_db"
    mv "$CDIR/ectd_db" "$CDIR/trailblazer_db"
fi
if [ -d "$CDIR/ectd_api" ]; then
    echo "Renaming ectd_api -> trailblazer_api"
    mv "$CDIR/ectd_api" "$CDIR/trailblazer_api"
fi
if [ -d "$CDIR/ectd_cli" ]; then
    echo "Renaming ectd_cli -> trailblazer_cli"
    mv "$CDIR/ectd_cli" "$CDIR/trailblazer_cli"
fi
if [ -d "$CDIR/ectd_app" ]; then
    echo "Renaming ectd_app -> trailblazer_app"
    mv "$CDIR/ectd_app" "$CDIR/trailblazer_app"
fi

# 2. Create New Crates
echo "Creating trailblazer_compliance and trailblazer_core..."
mkdir -p "$CDIR/trailblazer_compliance/src/validation"
mkdir -p "$CDIR/trailblazer_compliance/src/submission"
mkdir -p "$CDIR/trailblazer_core/src/inventory"

# 3. Migrate Logic
echo "Migrating Logic..."
if [ -d "$CDIR/ectd_core/src" ]; then
    cp -r "$CDIR/ectd_core/src/"* "$CDIR/trailblazer_compliance/src/validation/"
    rm -rf "$CDIR/ectd_core"
fi
if [ -d "$CDIR/ectd_service/src" ]; then
    cp -r "$CDIR/ectd_service/src/"* "$CDIR/trailblazer_compliance/src/submission/"
    rm -rf "$CDIR/ectd_service"
fi

# 4. Move Models to DB
echo "Moving Models to DB..."
mkdir -p "$CDIR/trailblazer_db/src/models/submission"
# Check if models exist in validation (from migration) and move them
if [ -d "$CDIR/trailblazer_compliance/src/validation/models" ]; then
    mv "$CDIR/trailblazer_compliance/src/validation/models/"* "$CDIR/trailblazer_db/src/models/submission/"
    rm -rf "$CDIR/trailblazer_compliance/src/validation/models"
fi

# 5. Populate SQL Schema
echo "Populating SQL Schema..."
SCHEMA_DIR="$CDIR/trailblazer_db/schema"
mkdir -p "$SCHEMA_DIR/foundation" "$SCHEMA_DIR/core" "$SCHEMA_DIR/inventory" "$SCHEMA_DIR/bridge" "$SCHEMA_DIR/submission" "$SCHEMA_DIR/content"

# Write SQL Files
cat <<EOF > "$SCHEMA_DIR/foundation/00_extensions.sql"
CREATE EXTENSION IF NOT EXISTS "pgcrypto";
EOF

cat <<EOF > "$SCHEMA_DIR/foundation/01_tenancy.sql"
CREATE TABLE clinics (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    subdomain VARCHAR(64) UNIQUE NOT NULL,
    config JSONB DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);
EOF

cat <<EOF > "$SCHEMA_DIR/core/01_departments.sql"
CREATE TABLE departments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    clinic_id UUID NOT NULL REFERENCES clinics(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    is_visible BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);
CREATE INDEX idx_departments_clinic ON departments(clinic_id);
EOF

cat <<EOF > "$SCHEMA_DIR/inventory/01_categories.sql"
CREATE TABLE inventory_categories (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    clinic_id UUID NOT NULL REFERENCES clinics(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    color VARCHAR(7),
    relevant_for_trials BOOLEAN DEFAULT FALSE,
    relevant_for_patients BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMPTZ DEFAULT NOW()
);
CREATE INDEX idx_inv_cat_clinic ON inventory_categories(clinic_id);
EOF

cat <<EOF > "$SCHEMA_DIR/inventory/02_items.sql"
CREATE TABLE inventory_items (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    clinic_id UUID NOT NULL REFERENCES clinics(id) ON DELETE CASCADE,
    category_id UUID NOT NULL REFERENCES inventory_categories(id),
    department_id UUID NOT NULL REFERENCES departments(id),
    parent_id UUID REFERENCES inventory_items(id),
    name VARCHAR(255) NOT NULL,
    quantity INTEGER NOT NULL DEFAULT 0,
    is_bookable BOOLEAN NOT NULL DEFAULT FALSE,
    max_overlapping_bookings INTEGER,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);
CREATE INDEX idx_inv_items_clinic ON inventory_items(clinic_id);
CREATE INDEX idx_inv_items_category ON inventory_items(category_id);
EOF

cat <<EOF > "$SCHEMA_DIR/bridge/01_submission_mappings.sql"
CREATE TABLE submission_mappings (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    submission_unit_id UUID NOT NULL REFERENCES submission_units(id),
    source_table VARCHAR(64) NOT NULL,
    source_id UUID NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(submission_unit_id, source_table, source_id)
);
EOF

cat <<EOF > "$SCHEMA_DIR/00_build_order.sql"
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
EOF

# 6. Create Source Files (lib.rs, mod.rs)
echo "Creating Rust modules..."

# trailblazer_db
echo "pub mod repository;" > "$CDIR/trailblazer_db/src/lib.rs"
echo "pub mod schema;" >> "$CDIR/trailblazer_db/src/lib.rs"
echo "pub mod models;" >> "$CDIR/trailblazer_db/src/lib.rs"

mkdir -p "$CDIR/trailblazer_db/src/models"
echo "pub mod submission;" > "$CDIR/trailblazer_db/src/models/mod.rs"
echo "pub mod inventory;" >> "$CDIR/trailblazer_db/src/models/mod.rs"

mkdir -p "$CDIR/trailblazer_db/src/models/submission"
echo "pub mod submission_unit;" > "$CDIR/trailblazer_db/src/models/submission/mod.rs"
echo "pub mod document;" >> "$CDIR/trailblazer_db/src/models/submission/mod.rs"
echo "pub mod context_of_use;" >> "$CDIR/trailblazer_db/src/models/submission/mod.rs"

# Inventory Model
cat <<EOF > "$CDIR/trailblazer_db/src/models/inventory.rs"
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
// use time::OffsetDateTime;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct InventoryItemDTO {
    pub id: Uuid,
    pub name: String,
    pub quantity: i32,
    pub category_name: String,
    pub department_name: String,
    pub is_bookable: bool,
    pub color: Option<String>,
}

pub struct InventoryQuery;

impl InventoryQuery {
    pub const FIND_ALL_BY_CLINIC: &'static str = r#"
        SELECT
            i.id, i.name, i.quantity, i.is_bookable,
            c.name as category_name, c.color,
            d.name as department_name
        FROM inventory_items i
        JOIN inventory_categories c ON i.category_id = c.id
        JOIN departments d ON i.department_id = d.id
        WHERE i.clinic_id = \$1
        ORDER BY i.name ASC
    "#;
}
EOF

# trailblazer_core
echo "pub mod inventory;" > "$CDIR/trailblazer_core/src/lib.rs"
mkdir -p "$CDIR/trailblazer_core/src/inventory"
echo "pub mod service;" > "$CDIR/trailblazer_core/src/inventory/mod.rs"
cat <<EOF > "$CDIR/trailblazer_core/src/inventory/service.rs"
use anyhow::{Result, Error};
use uuid::Uuid;

pub struct InventoryService {
}

impl InventoryService {
    pub async fn book_item(_item_id: Uuid, _quantity: i32) -> Result<(), Error> {
        Ok(())
    }
}
EOF

# trailblazer_compliance
echo "pub mod validation;" > "$CDIR/trailblazer_compliance/src/lib.rs"
echo "pub mod submission;" >> "$CDIR/trailblazer_compliance/src/lib.rs"

# 7. Update Imports (sed)
echo "Updating Imports..."
find "$CDIR" -type f -name "*.rs" -print0 | xargs -0 sed -i 's/ectd_db::/trailblazer_db::/g'
find "$CDIR" -type f -name "*.rs" -print0 | xargs -0 sed -i 's/ectd_core::models/trailblazer_db::models::submission/g'
find "$CDIR" -type f -name "*.rs" -print0 | xargs -0 sed -i 's/ectd_core::validation/trailblazer_compliance::validation/g'
find "$CDIR" -type f -name "*.rs" -print0 | xargs -0 sed -i 's/ectd_service::/trailblazer_compliance::submission::/g'
find "$CDIR" -type f -name "*.rs" -print0 | xargs -0 sed -i 's/ectd_core::/trailblazer_compliance::validation::/g'
find "$CDIR" -type f -name "*.rs" -print0 | xargs -0 sed -i 's/trailblazer_compliance::models/trailblazer_db::models::submission/g'
find "$CDIR" -type f -name "*.rs" -print0 | xargs -0 sed -i 's/crate::models/trailblazer_db::models::submission/g' # Fix internal refs in compliance

# Fix self-reference in DB
find "$CDIR/trailblazer_db" -type f -name "*.rs" -print0 | xargs -0 sed -i 's/trailblazer_db::/crate::/g'
find "$CDIR/trailblazer_db" -type f -name "*.rs" -print0 | xargs -0 sed -i 's/crate::models::submission/crate::models::submission/g' # Already correct?
# Fix specific double replacement if any
find "$CDIR" -type f -name "*.rs" -print0 | xargs -0 sed -i 's/trailblazer_db::models::submission::submission_unit::SubmissionUnit/trailblazer_db::models::submission::submission_unit::SubmissionUnit/g' # No-op check

# 8. Update docker-compose.yml
echo "Updating docker-compose.yml..."
sed -i 's|crates/ectd_db/schema|crates/trailblazer_db/schema|g' ectd_v4_forge/docker-compose.yml

echo "Refactor Complete."
