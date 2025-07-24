// use sqlx::{Row, mysql::MySqlRow};

// use crate::database::DbPool;
// use crate::entities::Board;
// use crate::repositories::boards::Boards;

// #[derive(Clone)]
// pub struct BoardsImpl {
//     pub pool: DbPool,
// }

// #[axum::async_trait]
// impl Boards for BoardsImpl {
//     async fn find(&self, id: i64) -> Option<Board> {
//         let row = sqlx::query("SELECT * FROM board WHERE id = ?")
//             .bind(id)
//             .fetch_optional(&*self.pool)
//             .await
//             .ok()??;
//         Some(row.into())
//     }

//     async fn find_by_title(&self, title: &str) -> Vec<Board> {
//         let rows = sqlx::query("SELECT * FROM board WHERE title = ?")
//             .bind(title)
//             .fetch_all(&*self.pool)
//             .await
//             .unwrap_or_else(|_| vec![]);
//         rows.into_iter().map(|row| row.into()).collect()
//     }

//     async fn find_by_user_id(&self, user_id: i64) -> Vec<Board> {
//         let rows = sqlx::query("SELECT * FROM board WHERE created_by = ? AND deleted = FALSE")
//             .bind(user_id)
//             .fetch_all(&*self.pool)
//             .await
//             .unwrap_or_else(|_| vec![]);
//         rows.into_iter().map(|row| row.into()).collect()
//     }

//     async fn find_by_board_id(&self, board_id: i64) -> Vec<Board> {
//         let rows = sqlx::query("SELECT * FROM board WHERE id = ? AND deleted = FALSE")
//             .bind(board_id)
//             .fetch_all(&*self.pool)
//             .await
//             .unwrap_or_else(|_| vec![]);
//         rows.into_iter().map(|row| row.into()).collect()
//     }

//     async fn store(&self, entity: &Board) -> Result<i64, String> {
//         let query = "INSERT INTO board (title, created_by) VALUES (?, ?)";
//         let result = sqlx::query(query)
//             .bind(&entity.title)
//             .bind(entity.created_by)
//             .execute(&*self.pool)
//             .await;

//         match result {
//              Ok(res) => Ok(res.last_insert_id()),
//             Err(e) => Err(format!("Failed to store board: {}", e)),
//         }
//     }

//     async fn update(&self, entity: &Board) -> Result<(), String> {
//         if let Some(id) = entity.id {
//             let query = "UPDATE board SET title = ?, updated_at = NOW() WHERE id = ?";
//             let result = sqlx::query(query)
//                 .bind(&entity.title)
//                 .bind(id)
//                 .execute(&*self.pool)
//                 .await;

//             match result {
//                 Ok(_) => Ok(()),
//                 Err(e) => Err(format!("Failed to update board: {}", e)),
//             }
//         } else {
//             Err("Board ID is not set".to_string())
//         }
//     }

//     async fn delete(&self, id: i64) -> Result<(), String> {
//         let query = "UPDATE board SET deleted = TRUE, updated_at = NOW() WHERE id = ?";
//         let result = sqlx::query(query).bind(id).execute(&*self.pool).await;

//         match result {
//             Ok(_) => Ok(()),
//             Err(e) => Err(format!("Failed to delete board: {}", e)),
//         }
//     }
// }

// impl From<MySqlRow> for Board {
//     fn from(row: MySqlRow) -> Self {
//         Board::new(
//             Some(row.get("id")),
//             row.get("title"),
//             row.get("created_by"),
//             row.get("created_at"),
//             row.get("updated_at"),
//         )
//     }
// }

use bb8::Pool;
use bb8_postgres::PostgresConnectionManager;
use std::sync::Arc;
use tokio_postgres::{NoTls, Row};

use crate::database::DbPool;
use crate::entities::Board;
use crate::repositories::boards::Boards;
use anyhow::Result;
use tokio_postgres::types::ToSql;

#[derive(Clone)]
pub struct BoardsImpl {
     pub pool: Arc<Pool<PostgresConnectionManager<NoTls>>>,
}

#[axum::async_trait]
impl Boards for BoardsImpl {
    async fn find(&self, id: i64) -> Result<Option<Board>, String> {
        let client = self.pool.get().await.map_err(|e| e.to_string())?;

        let row_opt = client
            .query_opt(
                "SELECT * FROM board WHERE id = $1",
                &[&(id as i64) as &(dyn ToSql + Sync)],
            )
            .await
            .map_err(|e| e.to_string())?;

        Ok(row_opt.map(|row| row_to_board(&row)))
    }

    async fn find_by_title(&self, title: &str) -> Result<Vec<Board>, String> {
        let client = self.pool.get().await.map_err(|e| e.to_string())?;

        let rows = client
            .query("SELECT * FROM board WHERE title = $1", &[&title])
            .await
            .map_err(|e| e.to_string())?;

        Ok(rows.into_iter().map(|r| row_to_board(&r)).collect())
    }

    async fn find_by_user_id(&self, user_id: i64) -> Result<Vec<Board>, String> {
        let client = self.pool.get().await.map_err(|e| e.to_string())?;
        let user_id_i64 = user_id as i64;

        let rows = client
            .query(
                "SELECT * FROM board WHERE created_by = $1 AND deleted = false",
                &[&user_id_i64],
            )
            .await
            .map_err(|e| e.to_string())?;

        Ok(rows.into_iter().map(|r| row_to_board(&r)).collect())
    }

    async fn find_by_board_id(&self, board_id: i64) -> Result<Vec<Board>, String> {
        let client = self.pool.get().await.map_err(|e| e.to_string())?;

        let rows = client
            .query(
                "SELECT * FROM board WHERE id = $1 AND deleted = false",
                &[&(board_id as i64)],
            )
            .await
            .map_err(|e| e.to_string())?;

        Ok(rows.into_iter().map(|r| row_to_board(&r)).collect())
    }

    async fn store(&self, entity: &Board) -> Result<i64, String> {
        let client = self.pool.get().await.map_err(|e| e.to_string())?;

        let row = client
            .query_one(
                "INSERT INTO board (title, created_by) VALUES ($1, $2) RETURNING id",
                &[&entity.title, &(entity.created_by as i64)],
            )
            .await
            .map_err(|e| e.to_string())?;

        let id: i64 = row.get("id");
        Ok(id as i64)
    }

    async fn update(&self, entity: &Board) -> Result<(), String> {
        if let Some(id) = entity.id {
            let client = self.pool.get().await.map_err(|e| e.to_string())?;

            client
                .execute(
                    "UPDATE board SET title = $1, updated_at = NOW() WHERE id = $2",
                    &[&entity.title, &(id as i64)],
                )
                .await
                .map_err(|e| e.to_string())?;

            Ok(())
        } else {
            Err("Board ID is not set".to_string())
        }
    }

    async fn delete(&self, id: i64) -> Result<(), String> {
        let client = self.pool.get().await.map_err(|e| e.to_string())?;

        client
            .execute(
                "UPDATE board SET deleted = TRUE, updated_at = NOW() WHERE id = $1",
                &[&(id as i64)],
            )
            .await
            .map_err(|e| e.to_string())?;

        Ok(())
    }
}

fn row_to_board(row: &Row) -> Board {
    let id: i64 = row.get("id");
    let created_by: i64 = row.get("created_by");

    Board::new(
        Some(id as i64),
        row.get("title"),
        created_by as i64,
        row.get("created_at"),
        row.get("updated_at"),
    )
}
