// use std::env;


// use sqlx::MySqlPool;
// use sqlx::mysql::MySqlPoolOptions;


// use crate::repos_impl::{AccountsImpl, BoardsImpl, TicketsImpl};
// use axum::extract::FromRef;

// pub type DbPool = Arc<MySqlPool>;

// use async_sqlx_session::MySqlSessionStore;



// #[derive(Clone)]
// pub struct Repositories {
//     pub accounts: AccountsImpl,
//     pub boards: BoardsImpl,
//     pub tickets: TicketsImpl,
// }

// impl FromRef<Repositories> for AccountsImpl {
//     fn from_ref(repos: &Repositories) -> Self {
//         repos.accounts.clone()
//     }
// }

// impl FromRef<Arc<Repositories>> for AccountsImpl {
//     fn from_ref(repos: &Arc<Repositories>) -> Self {
//         repos.accounts.clone()
//     }
// }

//Mysql接続
// pub async fn establish_connection() -> Repositories {
//     dotenv::dotenv().ok(); // .env 読み込み

//     let database_url = env::var(ENV_KEY_DATABASE_URL).expect("DATABASE_URL must be set");

//     // MySQLセッションストアの初期化
//     let store = MySqlSessionStore::new(&database_url)
//         .await
//         .expect("Failed to create MySQL session store");
//     store.migrate().await.expect("Migration failed");
//     store.spawn_cleanup_task(std::time::Duration::from_secs(3600));

//     // MySQL接続プールの作成
//     let pool = MySqlPoolOptions::new()
//         .max_connections(5)
//         .connect(&database_url)
//         .await
//         .expect("Failed to create MySQL connection pool");
//     let pool = Arc::new(pool);

//     Repositories {
//         accounts: AccountsImpl { pool: pool.clone() },
//         boards: BoardsImpl { pool: pool.clone() },
//         tickets: TicketsImpl { pool },
//     }
// }

// PostgreSQL接続
use std::sync::Arc;
use crate::constants::ENV_KEY_DATABASE_URL;
pub type DbPool = Pool<PostgresConnectionManager<NoTls>>;
use bb8::Pool;
use bb8_postgres::PostgresConnectionManager;
use tokio_postgres::NoTls;
use crate::repos_impl::{AccountsImpl, BoardsImpl, TicketsImpl};

#[derive(Clone)]
pub struct Repositories {
    pub accounts: AccountsImpl,
    pub boards: BoardsImpl,
    pub tickets: TicketsImpl,
}


pub async fn establish_connection() -> Repositories {
    dotenv::dotenv().ok(); // .env 読み込み

    let database_url = std::env::var(ENV_KEY_DATABASE_URL).expect("DATABASE_URL must be set");

    // PostgreSQL マネージャー作成
    let manager = PostgresConnectionManager::new_from_stringlike(database_url, NoTls)
        .expect("Failed to create Postgres manager");

    // bb8 プール作成
    let pool = Pool::builder()
        .build(manager)
        .await
        .expect("Failed to build connection pool");

    let pool = Arc::new(pool); // Arc に包む（必要なら）

    Repositories {
        accounts: AccountsImpl { pool: pool.clone() },
        boards: BoardsImpl { pool: pool.clone() },
        tickets: TicketsImpl { pool },
    }
}
