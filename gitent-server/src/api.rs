use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::{get, post},
    Router,
};
use gitent_core::{Change, ChangeType, Commit, CommitInfo, Session, Storage};
use serde::Deserialize;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

#[derive(Clone)]
pub struct AppState {
    pub storage: Arc<Mutex<Storage>>,
}

pub fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health_check))
        .route("/session", get(get_active_session))
        .route("/changes", get(get_uncommitted_changes))
        .route("/changes", post(create_change))
        .route("/commits", get(get_commits))
        .route("/commits", post(create_commit))
        .route("/commits/:id", get(get_commit))
        .with_state(state)
}

async fn health_check() -> impl IntoResponse {
    Json(serde_json::json!({"status": "ok"}))
}

async fn get_active_session(
    State(state): State<AppState>,
) -> Result<Json<Session>, (StatusCode, String)> {
    let storage = state.storage.lock().unwrap();
    storage
        .get_active_session()
        .map(Json)
        .map_err(|e| (StatusCode::NOT_FOUND, e.to_string()))
}

async fn get_uncommitted_changes(
    State(state): State<AppState>,
) -> Result<Json<Vec<Change>>, (StatusCode, String)> {
    let storage = state.storage.lock().unwrap();
    let session = storage
        .get_active_session()
        .map_err(|e| (StatusCode::NOT_FOUND, e.to_string()))?;

    storage
        .get_uncommitted_changes(&session.id)
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

#[derive(Deserialize)]
struct CreateChangeRequest {
    change_type: String,
    path: String,
    content_before: Option<String>,
    content_after: Option<String>,
    agent_id: Option<String>,
}

async fn create_change(
    State(state): State<AppState>,
    Json(req): Json<CreateChangeRequest>,
) -> Result<Json<Change>, (StatusCode, String)> {
    let storage = state.storage.lock().unwrap();
    let session = storage
        .get_active_session()
        .map_err(|e| (StatusCode::NOT_FOUND, e.to_string()))?;

    let change_type = ChangeType::parse(&req.change_type)
        .ok_or_else(|| (StatusCode::BAD_REQUEST, "Invalid change type".to_string()))?;

    let mut change = Change::new(change_type, std::path::PathBuf::from(req.path), session.id);

    if let Some(content) = req.content_before {
        change = change.with_content_before(content.into_bytes());
    }

    if let Some(content) = req.content_after {
        change = change.with_content_after(content.into_bytes());
    }

    if let Some(agent_id) = req.agent_id {
        change = change.with_agent_id(agent_id);
    }

    storage
        .create_change(&change)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(change))
}

async fn get_commits(
    State(state): State<AppState>,
) -> Result<Json<Vec<CommitInfo>>, (StatusCode, String)> {
    let storage = state.storage.lock().unwrap();
    let session = storage
        .get_active_session()
        .map_err(|e| (StatusCode::NOT_FOUND, e.to_string()))?;

    storage
        .get_commits_for_session(&session.id)
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

#[derive(Deserialize)]
struct CreateCommitRequest {
    message: String,
    agent_id: String,
    change_ids: Vec<String>,
}

async fn create_commit(
    State(state): State<AppState>,
    Json(req): Json<CreateCommitRequest>,
) -> Result<Json<Commit>, (StatusCode, String)> {
    let storage = state.storage.lock().unwrap();
    let session = storage
        .get_active_session()
        .map_err(|e| (StatusCode::NOT_FOUND, e.to_string()))?;

    let change_ids: Vec<Uuid> = req
        .change_ids
        .iter()
        .filter_map(|id| Uuid::parse_str(id).ok())
        .collect();

    let commit = Commit::new(req.message, req.agent_id, change_ids, session.id);

    storage
        .create_commit(&commit)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(commit))
}

async fn get_commit(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Commit>, (StatusCode, String)> {
    let commit_id =
        Uuid::parse_str(&id).map_err(|_| (StatusCode::BAD_REQUEST, "Invalid UUID".to_string()))?;

    let storage = state.storage.lock().unwrap();
    storage
        .get_commit(&commit_id)
        .map(Json)
        .map_err(|e| (StatusCode::NOT_FOUND, e.to_string()))
}
