
use crate::entities::Ticket;
use crate::repositories::tickets::Tickets;
use crate::request::UserContext;

//チケットすべて取得
pub async fn get_all_tickets(
    repo: &impl Tickets,
    user: &UserContext,
    board_id: i64,
) -> Result<Vec<Ticket>, String> {
    let tickets = repo.find_by_board_id(board_id).await?;
    Ok(tickets)
}

//チケット保存
pub async fn save_ticket(
    repo: &impl Tickets,
    user: &UserContext,
    ticket: Ticket,
) -> Result<(), String> {
    if ticket.id.is_some() {
        return Err("Ticket ID should not be set for new tickets".to_string());
    }
    repo.store(&ticket).await
}
//チケット更新
pub async fn update_ticket(
    repo: &impl Tickets,
    user: &UserContext,
    ticket: Ticket,
) -> Result<(), String> {
    if ticket.id.is_none() {
        return Err("Ticket ID is required for update".to_string());
    }
    repo.update(&ticket).await
}
//チケット削除
pub async fn delete_ticket(
    repo: &impl Tickets,
    user: &UserContext,
    ticket_id: i64,
) -> Result<(), String> {
    let ticket = repo
        .find(ticket_id)
        .await
        .ok_or_else(|| "Ticket not found".to_string())?;

    repo.delete(ticket_id).await
}
