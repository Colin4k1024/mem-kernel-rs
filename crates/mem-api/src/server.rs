//! Axum server and routes.

use axum::{
    extract::{Query, State},
    routing::{get, post},
    Json, Router,
};
use mem_scheduler::Scheduler;
use mem_types::MemCube;
use mem_types::{
    ApiAddRequest, ApiSearchRequest, AuditEvent, AuditEventKind, ForgetMemoryRequest,
    ForgetMemoryResponse, GetMemoryRequest, GetMemoryResponse, MemoryResponse, SearchResponse,
    SchedulerStatusResponse, UpdateMemoryRequest, UpdateMemoryResponse,
};
use serde::Deserialize;
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_http::cors::CorsLayer;
use uuid::Uuid;

pub struct AppState {
    pub cube: Arc<dyn MemCube + Send + Sync>,
    pub scheduler: Arc<dyn Scheduler + Send + Sync>,
    /// In-memory audit log (add/update/forget events).
    pub audit_log: Arc<RwLock<Vec<AuditEvent>>>,
}

pub fn router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/product/add", post(handle_add))
        .route("/product/search", post(handle_search))
        .route("/product/scheduler/status", get(handle_scheduler_status))
        .route("/product/update_memory", post(handle_update_memory))
        .route("/product/delete_memory", post(handle_delete_memory))
        .route("/product/get_memory", post(handle_get_memory))
        .route("/health", get(handle_health))
        .layer(CorsLayer::permissive())
        .with_state(state)
}

async fn push_audit(state: &AppState, event: AuditEvent) {
    state.audit_log.write().await.push(event);
}

async fn handle_add(
    State(state): State<Arc<AppState>>,
    Json(req): Json<ApiAddRequest>,
) -> Json<MemoryResponse> {
    if req.async_mode.as_str() == "async" {
        match state.scheduler.submit_add(req).await {
            Ok(task_id) => {
                tracing::info!(task_id = %task_id, "add job submitted (async)");
                Json(MemoryResponse {
                    code: 200,
                    message: "Memory add job submitted".to_string(),
                    data: Some(vec![serde_json::json!({ "task_id": task_id })]),
                })
            }
            Err(e) => Json(MemoryResponse {
                code: 500,
                message: e.to_string(),
                data: None,
            }),
        }
    } else {
        let cube_ids = req.writable_cube_ids();
        let user_id = req.user_id.clone();
        let cube_id = cube_ids.first().cloned().unwrap_or_else(|| user_id.clone());
        match state.cube.add_memories(&req).await {
            Ok(res) => {
                let memory_id = res
                    .data
                    .as_ref()
                    .and_then(|d| d.first())
                    .and_then(|v| v.get("id"))
                    .and_then(|v| v.as_str())
                    .map(String::from);
                push_audit(
                    &state,
                    AuditEvent {
                        event_id: Uuid::new_v4().to_string(),
                        kind: AuditEventKind::Add,
                        memory_id,
                        user_id,
                        cube_id,
                        timestamp: chrono::Utc::now().to_rfc3339(),
                        input_summary: None,
                        outcome: Some(format!("code={}", res.code)),
                    },
                )
                .await;
                Json(res)
            }
            Err(e) => Json(MemoryResponse {
                code: 500,
                message: e.to_string(),
                data: None,
            }),
        }
    }
}

async fn handle_search(
    State(state): State<Arc<AppState>>,
    Json(req): Json<ApiSearchRequest>,
) -> Json<SearchResponse> {
    match state.cube.search_memories(&req).await {
        Ok(res) => Json(res),
        Err(e) => Json(SearchResponse {
            code: 500,
            message: e.to_string(),
            data: None,
        }),
    }
}

#[derive(Debug, Deserialize)]
pub struct SchedulerStatusQuery {
    pub user_id: String,
    #[serde(default)]
    pub task_id: Option<String>,
}

async fn handle_scheduler_status(
    State(state): State<Arc<AppState>>,
    Query(q): Query<SchedulerStatusQuery>,
) -> Json<SchedulerStatusResponse> {
    let task_id = match &q.task_id {
        Some(t) => t.as_str(),
        None => {
            return Json(SchedulerStatusResponse {
                code: 400,
                message: "task_id is required".to_string(),
                data: None,
            });
        }
    };
    match state.scheduler.get_status(task_id).await {
        Ok(Some(job)) => Json(SchedulerStatusResponse {
            code: 200,
            message: "Success".to_string(),
            data: Some(job),
        }),
        Ok(None) => Json(SchedulerStatusResponse {
            code: 404,
            message: "Job not found".to_string(),
            data: None,
        }),
        Err(e) => Json(SchedulerStatusResponse {
            code: 500,
            message: e.to_string(),
            data: None,
        }),
    }
}

async fn handle_update_memory(
    State(state): State<Arc<AppState>>,
    Json(req): Json<UpdateMemoryRequest>,
) -> Json<UpdateMemoryResponse> {
    let user_id = req.user_id.clone();
    let cube_id = req.mem_cube_id.clone().unwrap_or_else(|| req.user_id.clone());
    let memory_id = req.memory_id.clone();
    match state.cube.update_memory(&req).await {
        Ok(res) => {
            push_audit(
                &state,
                AuditEvent {
                    event_id: Uuid::new_v4().to_string(),
                    kind: AuditEventKind::Update,
                    memory_id: Some(memory_id),
                    user_id,
                    cube_id,
                    timestamp: chrono::Utc::now().to_rfc3339(),
                    input_summary: None,
                    outcome: Some(format!("code={}", res.code)),
                },
            )
            .await;
            Json(res)
        }
        Err(e) => Json(UpdateMemoryResponse {
            code: 500,
            message: e.to_string(),
            data: None,
        }),
    }
}

async fn handle_delete_memory(
    State(state): State<Arc<AppState>>,
    Json(req): Json<ForgetMemoryRequest>,
) -> Json<ForgetMemoryResponse> {
    let user_id = req.user_id.clone();
    let cube_id = req.mem_cube_id.clone().unwrap_or_else(|| req.user_id.clone());
    let memory_id = req.memory_id.clone();
    match state.cube.forget_memory(&req).await {
        Ok(res) => {
            push_audit(
                &state,
                AuditEvent {
                    event_id: Uuid::new_v4().to_string(),
                    kind: AuditEventKind::Forget,
                    memory_id: Some(memory_id),
                    user_id,
                    cube_id,
                    timestamp: chrono::Utc::now().to_rfc3339(),
                    input_summary: None,
                    outcome: Some(format!("code={}", res.code)),
                },
            )
            .await;
            Json(res)
        }
        Err(e) => Json(ForgetMemoryResponse {
            code: 500,
            message: e.to_string(),
            data: None,
        }),
    }
}

async fn handle_get_memory(
    State(state): State<Arc<AppState>>,
    Json(req): Json<GetMemoryRequest>,
) -> Json<GetMemoryResponse> {
    match state.cube.get_memory(&req).await {
        Ok(res) => Json(res),
        Err(e) => Json(GetMemoryResponse {
            code: 500,
            message: e.to_string(),
            data: None,
        }),
    }
}

async fn handle_health() -> &'static str {
    "ok"
}
