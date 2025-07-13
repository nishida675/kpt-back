use crate::entities::Board;

#[axum::async_trait]
pub trait Boards {
    async fn find(&self, id: u64) -> Option<Board>; 
    async fn find_by_title(&self, title: &str) -> Vec<Board>;
    async fn find_by_user_id(&self, user_id: u64) -> Vec<Board>;
    async fn find_by_board_id(&self, board_id: u64) -> Vec<Board>;
    async fn store(&self, entity: &Board) -> Result<u64, String>;
    async fn update(&self, entity: &Board) -> Result<(), String>;
    async fn delete(&self, id: u64) -> Result<(), String>;
}
