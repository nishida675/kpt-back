use std::env;

use async_session::SessionStore;
use async_sqlx_session::PostgresSessionStore; 
use axum::{
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use axum_extra::headers::{Authorization, authorization::Bearer};
use axum_extra::TypedHeader;

use crate::constants::{AXUM_SESSION_COOKIE_NAME, AXUM_SESSION_USER_ID_KEY, ENV_KEY_DATABASE_URL};

#[derive(Deserialize, Serialize)]
pub struct UserContext {
    pub user_id: i64,
}

#[axum::async_trait]
impl<S> FromRequestParts<S> for UserContext
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, Json<serde_json::Value>);

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let error_response = || {
            (
                StatusCode::UNAUTHORIZED,
                Json(json!({ "error": "Unauthorized" })),
            )
        };

        dotenv::dotenv().ok();
        let database_url = env::var(ENV_KEY_DATABASE_URL).unwrap_or_default();

        // PostgreSQL用ストアを作成
        let store = PostgresSessionStore::new(&database_url)
            .await
            .map_err(|_| error_response())?;

        // AuthorizationヘッダーからBearerトークン取得
        let TypedHeader(Authorization(bearer)) =
            TypedHeader::<Authorization<Bearer>>::from_request_parts(parts, state)
                .await
                .map_err(|_| error_response())?;

        let token_str = bearer.token();

        // セッションロード
        let session = store
            .load_session(token_str.to_string())
            .await
            .map_err(|_| error_response())?
            .ok_or_else(error_response)?;

        // user_id取得
        let user_id = session
            .get::<i64>(AXUM_SESSION_USER_ID_KEY)
            .ok_or_else(error_response)?;

        Ok(UserContext { user_id })
    }
}
