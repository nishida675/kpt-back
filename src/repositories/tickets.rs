use crate::entities::Ticket;

#[axum::async_trait]
pub trait Tickets {
    async fn find(&self, id: i64) -> Option<Ticket>;
    async fn find_by_board_id(&self, board_id: i64) -> Result<Vec<Ticket>, String>;
    async fn store(&self, entity: &Ticket) -> Result<(), String>;
    async fn update(&self, entity: &Ticket) -> Result<(), String>;
    async fn delete(&self, id: i64) -> Result<(), String>;
}
