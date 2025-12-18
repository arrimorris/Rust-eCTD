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
        WHERE i.clinic_id = $1
        ORDER BY i.name ASC
    "#;
}
