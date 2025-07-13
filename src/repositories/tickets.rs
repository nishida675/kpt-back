use crate::entities::Ticket;

#[axum::async_trait]
pub trait Tickets {
    async fn find(&self, id: u64) -> Option<Ticket>;
    async fn find_by_board_id(&self, board_id: u64) -> Vec<Ticket>;
    async fn store(&self, entity: &Ticket) -> Result<(), String>;
    async fn update(&self, entity: &Ticket) -> Result<(), String>;
    async fn delete(&self, id: u64) -> Result<(), String>;
}