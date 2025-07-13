use crate::controllers::boards::BoardSummary;
use crate::entities::Board;
use crate::repositories::boards::Boards;
use crate::request::UserContext;
use axum::Json;

pub async fn get_all_boards(repo: &impl Boards, user: &UserContext) -> Json<Vec<BoardSummary>> {
    let boards = repo.find_by_user_id(user.user_id).await;

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
    board_id: u64,
) -> Result<Board, String> {
    let boards = repo.find_by_board_id(board_id).await;

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
) -> Result<u64, String> {
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
    board_id: u64,
) -> Result<(), String> {
    // board取得、なければエラーメッセージの文字列を返す
    let board = repo.find(board_id)
        .await
        .ok_or_else(|| "Board not found".to_string())?;

    // 作成者チェック
    if board.created_by != user.user_id {
        return Err("Unauthorized to delete this board".to_string());
    }

    // 削除処理。失敗したらエラーメッセージを文字列化して返す
    repo.delete(board_id).await
        .map_err(|e| format!("Failed to delete board: {}", e))
}
