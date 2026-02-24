//! MemOS REST API server: /product/add, /product/search, /product/scheduler/status, /health.

use mem_api::server;
use mem_cube::NaiveMemCube;
use mem_embed::OpenAiEmbedder;
use mem_graph::InMemoryGraphStore;
use mem_scheduler::InMemoryScheduler;
use mem_vec::{InMemoryVecStore, QdrantVecStore};
use std::net::SocketAddr;
use std::sync::Arc;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let cube: Arc<dyn mem_types::MemCube + Send + Sync> =
        if let Ok(url) = std::env::var("QDRANT_URL") {
            let store = QdrantVecStore::new(
                &url,
                std::env::var("QDRANT_COLLECTION").ok().as_deref(),
            )
            .map_err(|e| format!("QdrantVecStore: {}", e))?;
            tracing::info!("Using Qdrant vector store at {}", url);
            Arc::new(NaiveMemCube::new(
                InMemoryGraphStore::new(),
                store,
                OpenAiEmbedder::from_env(),
            ))
        } else {
            tracing::info!("Using in-memory vector store (set QDRANT_URL for Qdrant)");
            Arc::new(NaiveMemCube::new(
                InMemoryGraphStore::new(),
                InMemoryVecStore::new(None),
                OpenAiEmbedder::from_env(),
            ))
        };

    let audit_log = Arc::new(tokio::sync::RwLock::new(Vec::new()));
    let scheduler = Arc::new(InMemoryScheduler::new(
        Arc::clone(&cube),
        Some(Arc::clone(&audit_log)),
    ));
    let state = Arc::new(server::AppState {
        cube,
        scheduler,
        audit_log,
    });
    let app = server::router(state);
    let addr: SocketAddr = std::env::var("MEMOS_LISTEN")
        .unwrap_or_else(|_| "0.0.0.0:8001".to_string())
        .parse()?;
    tracing::info!("MemOS API listening on {}", addr);
    axum::serve(
        tokio::net::TcpListener::bind(addr).await?,
        app.into_make_service(),
    )
    .await?;
    Ok(())
}
