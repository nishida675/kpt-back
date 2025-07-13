use crate::database::Repositories;
use crate::repos_impl::BoardsImpl;
use crate::request::UserContext;
use crate::services;
use axum::Router;
use axum::http::StatusCode;
use axum::routing::{delete, get, post};
use axum::{
    extract::{Json, Path, State},
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use serde_json::ser;
use std::collections::HashSet;
use std::sync::Arc;

pub fn boards(repos: Arc<Repositories>) -> Router {
    Router::new()
        .route("/list", get(all_boards))
        .route("/save", post(save_board_tickets))
        .route("/data/:titleId", get(get_board_data))
        .route("/delete/:titleId", delete(delete_board)) // Assuming delete uses the same endpoint
        .with_state(repos)
}

async fn all_boards(
    user_ctx: UserContext,
    State(repos): State<Arc<Repositories>>,
) -> Json<Vec<BoardSummary>> {
    let boards_repo = &repos.boards;
    services::get_all_boards(boards_repo, &user_ctx).await
}

pub async fn save_board_tickets(
    user_ctx: UserContext,
    State(repos): State<Arc<Repositories>>,
    Json(payload): Json<SavePayload>,
) -> impl IntoResponse {
    let boards_repo = &repos.boards;
    let tickets_repo = &repos.tickets;

    if let Some(title_id_str) = payload.titleId.clone() {
        // titleIdがある場合は更新処理
        let title_id: u64 = match title_id_str.parse() {
            Ok(id) => id,
            Err(_) => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(ApiResponse {
                        message: "titleId is invalid".into(),
                        title: payload.title.clone(),
                    }),
                );
            }
        };

        match services::get_board_by_id(boards_repo, &user_ctx, title_id).await {
            Ok(mut board) => {
                // ボードのタイトル更新
                let update_result = services::update_board(
                    boards_repo,
                    &user_ctx,
                    &mut board,
                    payload.title.clone(),
                )
                .await;

                // boardの取得後、update_boardより前でも後でもOK
                let existing_tickets =
                    match services::get_all_tickets(tickets_repo, &user_ctx, title_id).await {
                        Ok(ts) => ts,
                        Err(e) => {
                            eprintln!("チケット取得失敗: {}", e);
                            return (
                                StatusCode::INTERNAL_SERVER_ERROR,
                                Json(ApiResponse {
                                    message: format!("Failed to fetch existing tickets: {}", e),
                                    title: payload.title.clone(),
                                }),
                            );
                        }
                    };

                // クライアント側から送られてきた有効なID一覧を収集
                let received_ids: HashSet<u64> = payload
                    .projectData
                    .lists
                    .iter()
                    .flat_map(|list| list.tickets.iter())
                    .filter_map(|ticket| ticket.id)
                    .filter(|id| *id != 0)
                    .collect();

                // DBにあるが、クライアントから来なかった → 削除
                for ticket in existing_tickets {
                    if let Some(ticket_id) = ticket.id {
                        if !received_ids.contains(&ticket_id) {
                            if let Err(e) =
                                services::delete_ticket(tickets_repo, &user_ctx, ticket_id).await
                            {
                                eprintln!("Failed to delete ticket {}: {}", ticket_id, e);
                            }
                        }
                    }
                }

                // チケット保存処理
                let mut ticket_errors = vec![];

                for list in &payload.projectData.lists {
                    for ticket in &list.tickets {
                        let save_result = if ticket.id == Some(0) {
                            // 新規チケット作成
                            let new_ticket = crate::entities::Ticket::create(
                                title_id,
                                user_ctx.user_id,
                                list.category.clone(),
                                ticket.content.clone(),
                            );
                            services::save_ticket(tickets_repo, &user_ctx, new_ticket).await
                        } else {
                            // 既存チケット更新
                            let updated_ticket = crate::entities::Ticket::new(
                                ticket.id,
                                title_id,
                                user_ctx.user_id,
                                list.category.clone(),
                                ticket.content.clone(),
                                chrono::Utc::now().naive_utc(),
                                chrono::Utc::now().naive_utc(),
                            );
                            services::update_ticket(tickets_repo, &user_ctx, updated_ticket).await
                        };

                        if let Err(e) = save_result {
                            ticket_errors.push(format!("Ticket {:?} failed: {}", ticket.id, e));
                        }
                    }
                }

                if let Err(e) = update_result {
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(ApiResponse {
                            message: format!("Board update failed: {}", e),
                            title: payload.title.clone(),
                        }),
                    );
                }

                if ticket_errors.is_empty() {
                    (
                        StatusCode::OK,
                        Json(ApiResponse {
                            message: "Board and tickets updated".into(),
                            title: payload.title.clone(),
                        }),
                    )
                } else {
                    (
                        StatusCode::OK,
                        Json(ApiResponse {
                            message: format!(
                                "Board updated but some tickets failed: {}",
                                ticket_errors.join("; ")
                            ),
                            title: payload.title.clone(),
                        }),
                    )
                }
            }
            Err(_) => (
                StatusCode::NOT_FOUND,
                Json(ApiResponse {
                    message: "Board not found".into(),
                    title: payload.title.clone(),
                }),
            ),
        }
    } else {
        // titleIdがない → 新規作成処理
        match services::save_board(boards_repo, &user_ctx, payload.title.clone()).await {
            Ok(board_id) => {
                let mut ticket_errors = vec![];

                for list in &payload.projectData.lists {
                    for ticket in &list.tickets {
                        let new_ticket = crate::entities::Ticket::create(
                            board_id,
                            user_ctx.user_id,
                            list.category.clone(),
                            ticket.content.clone(),
                        );

                        if let Err(e) =
                            services::save_ticket(tickets_repo, &user_ctx, new_ticket).await
                        {
                            ticket_errors
                                .push(format!("Failed to save ticket '{}': {}", ticket.content, e));
                        }
                    }
                }

                (
                    StatusCode::CREATED,
                    Json(ApiResponse {
                        message: if ticket_errors.is_empty() {
                            "Board and tickets created".into()
                        } else {
                            format!(
                                "Board created but some tickets failed: {}",
                                ticket_errors.join("; ")
                            )
                        },
                        title: payload.title.clone(),
                    }),
                )
            }
            Err(e) => {
                eprintln!("Create failed: {}", e);
                let response = (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiResponse {
                        message: format!("Create failed: {}", e),
                        title: payload.title.clone(),
                    }),
                );
                response
            }
        }
    }
}

pub async fn get_board_data(
    user_ctx: UserContext,
    Path(title_id): Path<u64>,
    State(repos): State<Arc<Repositories>>,
) -> Result<Json<BoardTicketSummary>, StatusCode> {
    let boards_repo = &repos.boards;
    let tickets_repo = &repos.tickets;

    // Board取得
    let board = match services::get_board_by_id(boards_repo, &user_ctx, title_id).await {
        Ok(b) => b,
        Err(e) => {
            eprintln!("Error fetching board data: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    // チケット取得
    let tickets = match services::get_all_tickets(tickets_repo, &user_ctx, title_id).await {
        Ok(ts) => ts,
        Err(e) => {
            eprintln!("Error fetching tickets: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    // カテゴリ別にチケットを分類（Keep / Problem / Try）
    let categories = vec!["Keep", "Problem", "Try"];
    let lists: Vec<List> = categories
        .into_iter()
        .map(|cat| {
            let list_tickets = tickets
                .iter()
                .filter(|t| t.category == cat)
                .map(|t| Ticket {
                    id: t.id,
                    content: t.content.clone(),
                })
                .collect();

            List {
                id: cat.to_string(),
                category: cat.to_string(),
                tickets: list_tickets,
            }
        })
        .collect();

    // 結果を組み立て
    let response = BoardTicketSummary {
        id: board.id.unwrap_or(0),
        title: board.title,
        projectData: ProjectData {
            id: board.id.map(|id| id.to_string()),
            lists,
        },
    };

    Ok(Json(response))
}

pub async fn delete_board(
    user_ctx: UserContext,
    Path(title_id): Path<u64>,
    State(repos): State<Arc<Repositories>>,
) -> Result<Response, StatusCode> {
    let boards_repo = &repos.boards;
    let tickets_repo = &repos.tickets;

    // チケットを先に削除

    match services::get_all_tickets(tickets_repo, &user_ctx, title_id).await {
        Ok(tickets) => {
            for ticket in tickets {
                if let Some(ticket_id) = ticket.id {
                    if let Err(e) =
                        services::delete_ticket(&repos.tickets, &user_ctx, ticket_id).await
                    {
                        eprintln!("Failed to delete ticket {}: {}", ticket_id, e);
                    }
                } else {
                    eprintln!("チケットにIDが存在しないため削除できません: {:?}", ticket);
                }
            }
        }
        Err(e) => {
            eprintln!("取得失敗: {}", e);
        }
    }

    //ボード削除
    match services::delete_board(boards_repo, &user_ctx, title_id).await {
        Ok(_) => Ok(Json(MessageResponse {
            message: "Board deleted successfully".into(),
        })
        .into_response()),
        Err(e) => {
            eprintln!("Error deleting board: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

#[derive(Serialize)]
pub struct BoardSummary {
    pub title: String,
    pub id: u64,
}

#[derive(Serialize)]
pub struct BoardTicketSummary {
    pub title: String,
    pub id: u64,
    pub projectData: ProjectData,
}

#[derive(Deserialize)]
pub struct SavePayload {
    pub projectData: ProjectData,
    pub title: String,
    pub titleId: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ticket {
    pub id: Option<u64>,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct List {
    pub id: String,
    pub category: String,
    pub tickets: Vec<Ticket>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectData {
    pub id: Option<String>,
    pub lists: Vec<List>,
}

#[derive(Serialize)]
struct ApiResponse {
    message: String,
    title: String,
}

#[derive(Serialize)]
struct MessageResponse {
    message: String,
}

#[derive(Debug, Deserialize)]
pub struct ProjectDetail {
    pub id: Option<String>,
    pub lists: Vec<List>,
}
