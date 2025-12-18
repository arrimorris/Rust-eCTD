use uuid::Uuid;
use sqlx::PgPool;
use sqlx::Row; // Added for .get()
use crate::error::{Result, Error};
use trailblazer_db::models::inventory::{InventoryItemDTO};

pub struct InventoryService {
    db: PgPool,
}

impl InventoryService {
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }

    /// The "Manager View": See everything in the clinic
    pub async fn list_items(&self, clinic_id: Uuid) -> Result<Vec<InventoryItemDTO>> {
        let items = sqlx::query_as::<_, InventoryItemDTO>(
            r#"
            SELECT
                i.id, i.name, i.quantity, i.is_bookable,
                c.name as category_name, c.color,
                d.name as department_name
            FROM inventory_items i
            JOIN inventory_categories c ON i.category_id = c.id
            JOIN departments d ON i.department_id = d.id
            WHERE i.clinic_id = $1
            ORDER BY i.name ASC
            "#
        )
        .bind(clinic_id)
        .fetch_all(&self.db)
        .await
        .map_err(|e| Error::Database(e.to_string()))?;

        Ok(items)
    }

    /// Business Logic: Book an Item
    /// This replaces the legacy Java "InventoryBooking" logic
    pub async fn book_item(&self, item_id: Uuid, _quantity: i32) -> Result<bool> {
        // 1. Check if bookable
        let item = sqlx::query(
            "SELECT is_bookable, quantity FROM inventory_items WHERE id = $1"
        )
        .bind(item_id)
        .fetch_optional(&self.db)
        .await
        .map_err(|e| Error::Database(e.to_string()))?
        .ok_or(Error::NotFound("Item not found".into()))?;

        let is_bookable: bool = item.get("is_bookable");

        if !is_bookable {
            return Err(Error::BusinessRule("Item is not bookable".into()));
        }

        // 2. (Future) Insert into booking table and check overlap
        Ok(true)
    }
}
