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
