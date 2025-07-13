use sqlx::{Row, mysql::MySqlRow};

use crate::database::DbPool;
use crate::entities::Ticket;
use crate::repositories::tickets::Tickets;

#[derive(Clone)]
pub struct TicketsImpl {
    pub pool: DbPool,
}

#[axum::async_trait]
impl Tickets for TicketsImpl {
    async fn find(&self, id: u64) -> Option<Ticket> {
        let row = sqlx::query("SELECT * FROM ticket WHERE id = ? AND deleted = FALSE")
            .bind(id)
            .fetch_optional(&*self.pool)
            .await
            .ok()??;
        Some(row.into())
    }

    async fn find_by_board_id(&self, board_id: u64) -> Vec<Ticket> {
        let rows = sqlx::query("SELECT * FROM ticket WHERE board_id = ? AND deleted = FALSE")
            .bind(board_id)
            .fetch_all(&*self.pool)
            .await
            .unwrap_or_else(|_| vec![]);
        rows.into_iter().map(|row| row.into()).collect()
    }

    async fn store(&self, entity: &Ticket) -> Result<(), String> {
        let query =
            "INSERT INTO ticket (board_id, author_id, category, content) VALUES (?, ?, ?, ?)";
        let result = sqlx::query(query)
            .bind(entity.board_id)
            .bind(entity.author_id)
            .bind(&entity.category)
            .bind(&entity.content)
            .execute(&*self.pool)
            .await;

        match result {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("Failed to store ticket: {}", e)),
        }
    }

    async fn update(&self, entity: &Ticket) -> Result<(), String> {
        if let Some(id) = entity.id {
            let query =
                "UPDATE ticket SET category = ?, content = ?, updated_at = NOW() WHERE id = ?";
            let result = sqlx::query(query)
                .bind(&entity.category)
                .bind(&entity.content)
                .bind(id)
                .execute(&*self.pool)
                .await;

            match result {
                Ok(_) => Ok(()),
                Err(e) => Err(format!("Failed to update ticket: {}", e)),
            }
        } else {
            Err("Ticket ID is not set".to_string())
        }
    }

    async fn delete(&self, id: u64) -> Result<(), String> {
        let query = "UPDATE ticket SET deleted = TRUE, updated_at = NOW() WHERE id = ?";
        let result = sqlx::query(query).bind(id).execute(&*self.pool).await;

        match result {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("Failed to delete ticket: {}", e)),
        }
    }
}

impl From<MySqlRow> for Ticket {
    fn from(row: MySqlRow) -> Self {
        Ticket::new(
            Some(row.get("id")),
            row.get("board_id"),
            row.get("author_id"),
            row.get("category"),
            row.get("content"),
            row.get("created_at"),
            row.get("updated_at"),
        )
    }
}
