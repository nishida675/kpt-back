use crate::controllers::boards::BoardSummary;
use crate::entities::Board;
use crate::repositories::boards::Boards;
use crate::request::UserContext;
use axum::Json;
use std::convert::TryInto;

pub async fn get_all_boards(repo: &impl Boards, user: &UserContext) -> Json<Vec<BoardSummary>> {
    let user_id = user.user_id.try_into().unwrap();
    let boards = repo.find_by_user_id(user_id).await.unwrap();

    let summaries = boards
        .into_iter()
        .map(|board| BoardSummary {
            id: board.id.unwrap_or(0),
            title: board.title.clone(),
        })
        .collect();

    Json(summaries)
}

pub async fn get_board_by_id(
    repo: &impl Boards,
    user: &UserContext,
    board_id: i64,
) -> Result<Board, String> {
    let boards = repo.find_by_board_id(board_id).await?; // Result を ? で処理

    if let Some(board) = boards.into_iter().next() {
        if board.created_by == user.user_id || !board.is_deleted() {
            return Ok(board);
        }
    }

    Err("Board not found or access denied".to_string())
}

pub async fn save_board(
    repo: &impl Boards,
    user: &UserContext,
    title: String,
) -> Result<i64, String> {
    let mut board = Board::create(title, user.user_id);
    let bored_id = repo.store(&board).await?;
    board.id = Some(bored_id);
    Ok(board.id.unwrap())
}

pub async fn update_board(
    repo: &impl Boards,
    user: &UserContext,
    board: &mut Board,
    new_title: String,
) -> Result<(), String> {
    if board.id.is_none() {
        return Err("Board ID is required".to_string());
    }
    if board.created_by != user.user_id {
        return Err("Unauthorized to update this board".to_string());
    }
    board.update(new_title);
    repo.update(board).await
}

pub async fn delete_board(
    repo: &impl Boards,
    user: &UserContext,
    board_id: i64,
) -> Result<(), String> {
    let board = repo
        .find(board_id)
        .await
        .map_err(|e| format!("DB error: {}", e))?
        .ok_or_else(|| "Board not found".to_string())?;

    if board.created_by != user.user_id {
        return Err("Unauthorized to delete this board".to_string());
    }

    repo.delete(board_id)
        .await
        .map_err(|e| format!("Failed to delete board: {}", e))
}
