use crate::entities::Board;

#[axum::async_trait]
#[axum::async_trait]
pub trait Boards {
    async fn find(&self, id: i64) -> Result<Option<Board>, String>;
    async fn find_by_title(&self, title: &str) -> Result<Vec<Board>, String>;
    async fn find_by_user_id(&self, user_id: i64) -> Result<Vec<Board>, String>;
    async fn find_by_board_id(&self, board_id: i64) -> Result<Vec<Board>, String>;
    async fn store(&self, entity: &Board) -> Result<i64, String>;
    async fn update(&self, entity: &Board) -> Result<(), String>;
    async fn delete(&self, id: i64) -> Result<(), String>;
}

