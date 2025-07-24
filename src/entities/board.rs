use chrono::{NaiveDateTime, Utc};
use serde::Serialize;

#[derive(Serialize, Debug, Clone)]
pub struct Board {
    pub id: Option<i64>,
    pub title: String,
    pub created_by: i64,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    deleted: bool,
}

impl Board {
    // DBなどからの読み込み時
    pub fn new(
        id: Option<i64>,
        title: String,
        created_by: i64,
        created_at: NaiveDateTime,
        updated_at: NaiveDateTime,
    ) -> Board {
        Board {
            id,
            title,
            created_by,
            created_at,
            updated_at,
            deleted: false,
        }
    }

    // 新規作成用
    pub fn create(title: String, created_by: i64) -> Board {
        let now = Utc::now().naive_utc();
        Board {
            id: None,
            title,
            created_by,
            created_at: now,
            updated_at: now,
            deleted: false,
        }
    }

    // 更新（タイトル変更など）
    pub fn update(&mut self, new_title: String) {
        self.title = new_title;
        self.updated_at = Utc::now().naive_utc();
    }

    pub fn id(&self) -> Option<i64> {
        self.id
    }

    pub fn is_deleted(&self) -> bool {
        self.deleted
    }

    pub fn delete(&mut self) {
        self.deleted = true;
        self.updated_at = Utc::now().naive_utc();
    }

    pub fn title(&self) -> &str {
        &self.title
    }
}
