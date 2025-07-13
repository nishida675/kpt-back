use chrono::{NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ticket {
    pub id: Option<u64>,
    pub board_id: u64,
    pub author_id: u64,
    pub category: String,
    pub content: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    deleted: bool,
}

impl Ticket {
    pub fn new(
        id: Option<u64>,
        board_id: u64,
        author_id: u64,
        category: String,
        content: String,
        created_at: NaiveDateTime,
        updated_at: NaiveDateTime,
    ) -> Ticket {
        Ticket {
            id,
            board_id,
            author_id,
            category,
            content,
            created_at,
            updated_at,
            deleted: false,
        }
    }
    pub fn create(
        board_id: u64,
        author_id: u64,
        category: String,
        content: String,
    ) -> Ticket {
        Ticket {
            id:None,
            board_id,
            author_id,
            category,
            content,
            created_at: Utc::now().naive_utc(),
            updated_at: Utc::now().naive_utc(),
            deleted: false,
        }
    }

    pub fn update(&mut self, new_category: String, new_content: String) {
        self.category = new_category;
        self.content = new_content;
        self.updated_at = Utc::now().naive_utc();
    }

    pub fn id(&self) -> Option<u64> {
        self.id
    }

    pub fn is_deleted(&self) -> bool {
        self.deleted
    }

    pub fn delete(&mut self) {
        self.deleted = true;
    }
}
