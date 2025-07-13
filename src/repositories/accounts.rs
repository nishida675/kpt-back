use std::collections::{HashMap, HashSet};
use crate::entities::Account;

#[axum::async_trait]
pub trait Accounts {
    async fn find(&self, ids: HashSet<u64>) -> HashMap<u64, Account>;
    async fn find_by(&self, display_name: &str) -> Option<Account>;
    async fn store(&self, entity: &Account);
}