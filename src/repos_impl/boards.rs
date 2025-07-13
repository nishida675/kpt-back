use sqlx::{Row, mysql::MySqlRow};

use crate::database::DbPool;
use crate::entities::Board;
use crate::repositories::boards::Boards;

#[derive(Clone)]
pub struct BoardsImpl {
    pub pool: DbPool,
}

#[axum::async_trait]
impl Boards for BoardsImpl {
    async fn find(&self, id: u64) -> Option<Board> {
        let row = sqlx::query("SELECT * FROM board WHERE id = ?")
            .bind(id)
            .fetch_optional(&*self.pool)
            .await
            .ok()??;
        Some(row.into())
    }

    async fn find_by_title(&self, title: &str) -> Vec<Board> {
        let rows = sqlx::query("SELECT * FROM board WHERE title = ?")
            .bind(title)
            .fetch_all(&*self.pool)
            .await
            .unwrap_or_else(|_| vec![]);
        rows.into_iter().map(|row| row.into()).collect()
    }

    async fn find_by_user_id(&self, user_id: u64) -> Vec<Board> {
        let rows = sqlx::query("SELECT * FROM board WHERE created_by = ? AND deleted = FALSE")
            .bind(user_id)
            .fetch_all(&*self.pool)
            .await
            .unwrap_or_else(|_| vec![]);
        rows.into_iter().map(|row| row.into()).collect()
    }

    async fn find_by_board_id(&self, board_id: u64) -> Vec<Board> {
        let rows = sqlx::query("SELECT * FROM board WHERE id = ? AND deleted = FALSE")
            .bind(board_id)
            .fetch_all(&*self.pool)
            .await
            .unwrap_or_else(|_| vec![]);
        rows.into_iter().map(|row| row.into()).collect()
    }

    async fn store(&self, entity: &Board) -> Result<u64, String> {
        let query = "INSERT INTO board (title, created_by) VALUES (?, ?)";
        let result = sqlx::query(query)
            .bind(&entity.title)
            .bind(entity.created_by)
            .execute(&*self.pool)
            .await;

        match result {
             Ok(res) => Ok(res.last_insert_id()),
            Err(e) => Err(format!("Failed to store board: {}", e)),
        }
    }

    async fn update(&self, entity: &Board) -> Result<(), String> {
        if let Some(id) = entity.id {
            let query = "UPDATE board SET title = ?, updated_at = NOW() WHERE id = ?";
            let result = sqlx::query(query)
                .bind(&entity.title)
                .bind(id)
                .execute(&*self.pool)
                .await;

            match result {
                Ok(_) => Ok(()),
                Err(e) => Err(format!("Failed to update board: {}", e)),
            }
        } else {
            Err("Board ID is not set".to_string())
        }
    }

    async fn delete(&self, id: u64) -> Result<(), String> {
        let query = "UPDATE board SET deleted = TRUE, updated_at = NOW() WHERE id = ?";
        let result = sqlx::query(query).bind(id).execute(&*self.pool).await;

        match result {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("Failed to delete board: {}", e)),
        }
    }
}

impl From<MySqlRow> for Board {
    fn from(row: MySqlRow) -> Self {
        Board::new(
            Some(row.get("id")),
            row.get("title"),
            row.get("created_by"),
            row.get("created_at"),
            row.get("updated_at"),
        )
    }
}
