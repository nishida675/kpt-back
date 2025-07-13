use crate::database::Repositories;
use crate::repos_impl::AccountsImpl;
use crate::services::{self, SessionToken};
use axum::extract::Json;
use axum::extract::State;
use axum::http::StatusCode;
use axum::{
    Router,
    extract::Form,
    response::{AppendHeaders, IntoResponse, Redirect},
    routing,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;

pub fn accounts(repos: Arc<Repositories>) -> Router {
    Router::new()
        .route("/new", routing::post(post))
        .route("/session", routing::post(api_login))
        .with_state(repos)
}

async fn post(
    State(accounts_repo): State<AccountsImpl>,
    Json(payload): Json<SignUpForm>,
) -> impl IntoResponse {
    services::create_account(&accounts_repo, &payload.password, &payload.display_name).await;

    (
        StatusCode::CREATED,
        Json(ApiResponse {
            message: "Account created successfully",
        }),
    )
}

async fn new_session(
    State(accounts_repo): State<AccountsImpl>,
    form: Form<SignInForm>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    let session_token =
        services::create_session(&accounts_repo, &form.display_name, &form.password).await;
    redirect_with_session(session_token)
}

fn redirect_with_session(
    session: Option<SessionToken>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    if let Some(session_token) = session {
        let headers = AppendHeaders(vec![("Set-Cookie", session_token.cookie())]); // レスポンスヘッダーの作成
        let response = Redirect::to("/");
        Ok((headers, response))
    } else {
        Err(Redirect::to("/login?error=invalid"))
    }
}

async fn api_login(
    State(accounts_repo): State<AccountsImpl>,
    Json(payload): Json<SignInForm>,
) -> impl IntoResponse {
    let session_token =
        services::create_session(&accounts_repo, &payload.display_name, &payload.password).await;

    if let Some(token) = session_token {
        Json(json!({
            "message": "ログイン成功",
            "token": token.value(),
        }))
        .into_response()
    } else {
        (
            StatusCode::UNAUTHORIZED,
            Json(json!({ "message": "ログイン失敗" })),
        )
            .into_response()
    }
}

#[derive(Deserialize)]
struct SignInForm {
    display_name: String,
    password: String,
}

#[derive(Deserialize)]
struct SignUpForm {
    display_name: String,
    password: String,
}

#[derive(Serialize)]
struct SignInResponse {
    token: String,
}

#[derive(Serialize)]
struct ApiResponse<'a> {
    message: &'a str,
}
