use sqlx::{Row, mysql::MySqlRow};
use std::collections::{HashMap, HashSet};

use crate::database::DbPool;
use crate::entities::Account;
use crate::repositories::accounts::Accounts;

#[derive(Clone)]
pub struct AccountsImpl {
    pub pool: DbPool,
}

#[axum::async_trait]
impl Accounts for AccountsImpl {
    async fn find(&self, ids: HashSet<u64>) -> HashMap<u64, Account> {
        if ids.is_empty() {
            return HashMap::new();
        }

        let mut query = String::from("SELECT * FROM accounts WHERE id IN (");
        let placeholders = vec!["?"; ids.len()].join(",");
        query.push_str(&placeholders);
        query.push(')');

        let mut sql = sqlx::query(&query);
        for id in &ids {
            sql = sql.bind(id);
        }

        let rows = sqlx::query(&query)
            .fetch_all(self.pool.as_ref())
            .await
            .unwrap();

        rows.into_iter()
            .map(|x| {
                let account: Account = x.into();
                (account.id().unwrap(), account)
            })
            .collect()
    }

    async fn find_by(&self, display_name: &str) -> Option<Account> {
        let row = sqlx::query("SELECT * FROM accounts WHERE display_name = ?")
            .bind(display_name)
            .fetch_optional(&*self.pool)
            .await
            .ok()??;

        Some(row.into())
    }

    async fn store(&self, entity: &Account) {
        sqlx::query("INSERT INTO accounts (password, display_name) VALUES (?, ?)")
            .bind(&entity.hashed_password)
            .bind(&entity.display_name)
            .execute(&*self.pool)
            .await
            .ok();
    }
}

impl From<MySqlRow> for Account {
    fn from(r: MySqlRow) -> Self {
        Account::new(
            Some(r.get::<u64, _>("id")),
            r.get("password"),
            r.get("display_name"),
        )
    }
}