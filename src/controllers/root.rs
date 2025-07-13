use std::sync::Arc;
use axum::{Router, routing::{options}};
use crate::database;
use tower_http::cors::CorsLayer;
use axum::http::{HeaderValue, Method, header};
use crate::controllers::accounts;
use crate::controllers::boards;

pub async fn app() -> Router {
    let repos = Arc::new(database::establish_connection().await);

    let cors = CorsLayer::new()
        .allow_origin(HeaderValue::from_static("http://localhost:3000"))
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers([
            header::CONTENT_TYPE,
            header::AUTHORIZATION,
        ])
        .allow_credentials(true);

    Router::new()
        .route("/accounts/session", options(|| async {}))
        .route("/boards/list", options(|| async {}))
        .nest("/accounts", accounts::accounts(repos.clone()))
        .nest("/boards", boards::boards(repos.clone()))
        .layer(cors)
}
