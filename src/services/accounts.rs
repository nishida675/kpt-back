use std::env;
use std::time::Duration;

use async_session::{Session, SessionStore};
use async_sqlx_session::MySqlSessionStore;

use crate::constants::{AXUM_SESSION_COOKIE_NAME, AXUM_SESSION_USER_ID_KEY, ENV_KEY_DATABASE_URL};


use crate::entities::Account;
use crate::repositories::accounts::Accounts;

pub async fn create_account(repo: &impl Accounts, password: &str, display_name: &str) {
    let new_account = Account::create( password, display_name);
    repo.store(&new_account).await;
}

pub async fn create_session(repo: &impl Accounts, display_name: &str, password: &str) -> Option<SessionToken> {
    let account = repo.find_by(display_name).await;
    if let Some(account) = account {
        if !account.matches_password(password) {
            return None;
        }

        dotenv::dotenv().ok();
        let database_url = env::var(ENV_KEY_DATABASE_URL).unwrap();
        let store = MySqlSessionStore::new(&database_url).await.unwrap();

        let mut session = Session::new();
        session
            .insert(AXUM_SESSION_USER_ID_KEY, account.id().unwrap())
            .unwrap();
        session.expire_in(Duration::from_secs(604800));

        let cookie = store.store_session(session).await.unwrap().unwrap();

        Some(SessionToken(cookie))
    } else {
        None
    }
}

pub struct SessionToken(String);

impl SessionToken {
    pub fn cookie(&self) -> String {
        format!(
            "{}={}; Max-Age=604800; Path=/; HttpOnly",
            AXUM_SESSION_COOKIE_NAME, &self.0
        )
    }
    pub fn value(&self) -> &str {
    &self.0
}
}