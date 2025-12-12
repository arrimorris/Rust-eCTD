use anyhow::{Result, Error};
use uuid::Uuid;

pub struct InventoryService {
}

impl InventoryService {
    pub async fn book_item(_item_id: Uuid, _quantity: i32) -> Result<(), Error> {
        Ok(())
    }
}
