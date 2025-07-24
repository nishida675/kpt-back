// use sqlx::{Row, mysql::MySqlRow};

// use crate::database::DbPool;
// use crate::entities::Ticket;
// use crate::repositories::tickets::Tickets;

// #[derive(Clone)]
// pub struct TicketsImpl {
//     pub pool: DbPool,
// }

// #[axum::async_trait]
// impl Tickets for TicketsImpl {
//     async fn find(&self, id: i64) -> Option<Ticket> {
//         let row = sqlx::query("SELECT * FROM ticket WHERE id = ? AND deleted = FALSE")
//             .bind(id)
//             .fetch_optional(&*self.pool)
//             .await
//             .ok()??;
//         Some(row.into())
//     }

//     async fn find_by_board_id(&self, board_id: i64) -> Vec<Ticket> {
//         let rows = sqlx::query("SELECT * FROM ticket WHERE board_id = ? AND deleted = FALSE")
//             .bind(board_id)
//             .fetch_all(&*self.pool)
//             .await
//             .unwrap_or_else(|_| vec![]);
//         rows.into_iter().map(|row| row.into()).collect()
//     }

//     async fn store(&self, entity: &Ticket) -> Result<(), String> {
//         let query =
//             "INSERT INTO ticket (board_id, author_id, category, content) VALUES (?, ?, ?, ?)";
//         let result = sqlx::query(query)
//             .bind(entity.board_id)
//             .bind(entity.author_id)
//             .bind(&entity.category)
//             .bind(&entity.content)
//             .execute(&*self.pool)
//             .await;

//         match result {
//             Ok(_) => Ok(()),
//             Err(e) => Err(format!("Failed to store ticket: {}", e)),
//         }
//     }

//     async fn update(&self, entity: &Ticket) -> Result<(), String> {
//         if let Some(id) = entity.id {
//             let query =
//                 "UPDATE ticket SET category = ?, content = ?, updated_at = NOW() WHERE id = ?";
//             let result = sqlx::query(query)
//                 .bind(&entity.category)
//                 .bind(&entity.content)
//                 .bind(id)
//                 .execute(&*self.pool)
//                 .await;

//             match result {
//                 Ok(_) => Ok(()),
//                 Err(e) => Err(format!("Failed to update ticket: {}", e)),
//             }
//         } else {
//             Err("Ticket ID is not set".to_string())
//         }
//     }

//     async fn delete(&self, id: i64) -> Result<(), String> {
//         let query = "UPDATE ticket SET deleted = TRUE, updated_at = NOW() WHERE id = ?";
//         let result = sqlx::query(query).bind(id).execute(&*self.pool).await;

//         match result {
//             Ok(_) => Ok(()),
//             Err(e) => Err(format!("Failed to delete ticket: {}", e)),
//         }
//     }
// }

// impl From<MySqlRow> for Ticket {
//     fn from(row: MySqlRow) -> Self {
//         Ticket::new(
//             Some(row.get("id")),
//             row.get("board_id"),
//             row.get("author_id"),
//             row.get("category"),
//             row.get("content"),
//             row.get("created_at"),
//             row.get("updated_at"),
//         )
//     }
// }

use bb8::Pool;
use bb8_postgres::PostgresConnectionManager;
use std::sync::Arc;
use tokio_postgres::{NoTls, Row, types::ToSql};

use crate::database::DbPool;
use crate::entities::Ticket;
use crate::repositories::tickets::Tickets;
use anyhow::Result;

#[derive(Clone)]
pub struct TicketsImpl {
    pub pool: Arc<Pool<PostgresConnectionManager<NoTls>>>,
}

#[axum::async_trait]
impl Tickets for TicketsImpl {
    async fn find(&self, id: i64) -> Option<Ticket> {
        let client = self.pool.get().await.ok()?;

        let row = client
            .query_opt(
                "SELECT * FROM ticket WHERE id = $1 AND deleted = FALSE",
                &[&(id as i64)],
            )
            .await
            .ok()??;

        Some(row_to_ticket(&row))
    }

    async fn find_by_board_id(&self, board_id: i64) -> Result<Vec<Ticket>, String> {
        let client = self.pool.get().await.map_err(|e| e.to_string())?;
        let rows = client
            .query(
                "SELECT * FROM ticket WHERE board_id = $1 AND deleted = FALSE",
                &[&(board_id as i64)],
            )
            .await
            .map_err(|e| e.to_string())?;

        Ok(rows.into_iter().map(|row| row_to_ticket(&row)).collect())
    }

    async fn store(&self, entity: &Ticket) -> Result<(), String> {
        let client = self.pool.get().await.map_err(|e| e.to_string())?;

        let result = client
            .execute(
                "INSERT INTO ticket (board_id, author_id, category, content) VALUES ($1, $2, $3, $4)",
                &[
                    &(entity.board_id as i64),
                    &(entity.author_id as i64),
                    &entity.category,
                    &entity.content,
                ],
            )
            .await;

        match result {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("Failed to store ticket: {}", e)),
        }
    }

    async fn update(&self, entity: &Ticket) -> Result<(), String> {
        if let Some(id) = entity.id {
            let client = self.pool.get().await.map_err(|e| e.to_string())?;

            let result = client
                .execute(
                    "UPDATE ticket SET category = $1, content = $2, updated_at = NOW() WHERE id = $3",
                    &[&entity.category, &entity.content, &(id as i64)],
                )
                .await;

            match result {
                Ok(_) => Ok(()),
                Err(e) => Err(format!("Failed to update ticket: {}", e)),
            }
        } else {
            Err("Ticket ID is not set".to_string())
        }
    }

    async fn delete(&self, id: i64) -> Result<(), String> {
        let client = self.pool.get().await.map_err(|e| e.to_string())?;
        let result = client
            .execute(
                "UPDATE ticket SET deleted = TRUE, updated_at = NOW() WHERE id = $1",
                &[&(id as i64)],
            )
            .await;

        match result {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("Failed to delete ticket: {}", e)),
        }
    }
}

fn row_to_ticket(row: &Row) -> Ticket {
    let id: i64 = row.get("id");
    let board_id: i64 = row.get("board_id");
    let author_id: i64 = row.get("author_id");

    Ticket::new(
        Some(id as i64),
        board_id as i64,
        author_id as i64,
        row.get("category"),
        row.get("content"),
        row.get("created_at"),
        row.get("updated_at"),
    )
}
