// use sqlx::{Row, mysql::MySqlRow};
// use std::collections::{HashMap, HashSet};

// use crate::database::DbPool;
// use crate::entities::Account;
// use crate::repositories::accounts::Accounts;

// #[derive(Clone)]
// pub struct AccountsImpl {
//     pub pool: DbPool,
// }

// #[axum::async_trait]
// impl Accounts for AccountsImpl {
//     async fn find(&self, ids: HashSet<u64>) -> HashMap<u64, Account> {
//         if ids.is_empty() {
//             return HashMap::new();
//         }

//         let mut query = String::from("SELECT * FROM accounts WHERE id IN (");
//         let placeholders = vec!["?"; ids.len()].join(",");
//         query.push_str(&placeholders);
//         query.push(')');

//         let mut sql = sqlx::query(&query);
//         for id in &ids {
//             sql = sql.bind(id);
//         }

//         let rows = sqlx::query(&query)
//             .fetch_all(self.pool.as_ref())
//             .await
//             .unwrap();

//         rows.into_iter()
//             .map(|x| {
//                 let account: Account = x.into();
//                 (account.id().unwrap(), account)
//             })
//             .collect()
//     }

//     async fn find_by(&self, display_name: &str) -> Option<Account> {
//         let row = sqlx::query("SELECT * FROM accounts WHERE display_name = ?")
//             .bind(display_name)
//             .fetch_optional(&*self.pool)
//             .await
//             .ok()??;

//         Some(row.into())
//     }

//     async fn store(&self, entity: &Account) {
//         sqlx::query("INSERT INTO accounts (password, display_name) VALUES (?, ?)")
//             .bind(&entity.hashed_password)
//             .bind(&entity.display_name)
//             .execute(&*self.pool)
//             .await
//             .ok();
//     }
// }

// impl From<MySqlRow> for Account {
//     fn from(r: MySqlRow) -> Self {
//         Account::new(
//             Some(r.get::<u64, _>("id")),
//             r.get("password"),
//             r.get("display_name"),
//         )
//     }
// }

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use bb8::Pool;
use bb8_postgres::PostgresConnectionManager;
use tokio_postgres::{NoTls, Row};

use crate::entities::Account;
use crate::repositories::accounts::Accounts;

#[derive(Clone)]
pub struct AccountsImpl {
    pub pool: Arc<Pool<PostgresConnectionManager<NoTls>>>,
}

#[axum::async_trait]
impl Accounts for AccountsImpl {
    async fn find(&self, ids: HashSet<i64>) -> HashMap<i64, Account> {
        if ids.is_empty() {
            return HashMap::new();
        }

        let conn = self.pool.get().await.unwrap();

        let placeholders: Vec<String> = (1..=ids.len()).map(|i| format!("${}", i)).collect();
        let query = format!(
            "SELECT * FROM accounts WHERE id IN ({})",
            placeholders.join(",")
        );

        let params: Vec<i64> = ids.iter().map(|id| *id as i64).collect();
        let params_refs: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> =
            params.iter().map(|id| id as _).collect();

        let rows = conn.query(&query, &params_refs[..]).await.unwrap();

        rows.into_iter()
            .map(|row| {
                let account = row_to_account(&row);
                (account.id().unwrap(), account)
            })
            .collect()
    }

    async fn find_by(&self, display_name: &str) -> Option<Account> {
        let conn = self.pool.get().await.ok()?;
        let row = conn
            .query_opt(
                "SELECT * FROM accounts WHERE display_name = $1",
                &[&display_name],
            )
            .await
            .ok()??;

        Some(row_to_account(&row))
    }

    async fn store(&self, entity: &Account) {
        let conn = self.pool.get().await.ok();
        if let Some(client) = conn {
            let _ = client
                .execute(
                    "INSERT INTO accounts (password, display_name) VALUES ($1, $2)",
                    &[&entity.hashed_password, &entity.display_name],
                )
                .await.expect("Failed to execute insert");
        }
    }
}

fn row_to_account(row: &Row) -> Account {
    let id: i64 = row.get("id");
    Account::new(
        Some(id as i64),
        row.get("password"),
        row.get("display_name"),
    )
}
