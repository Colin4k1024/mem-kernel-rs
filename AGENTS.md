# Agent Guidelines for mem-kernel-rs

This file contains guidelines for agentic coding agents working in this repository.

## Build, Lint, and Test Commands

### Local Development

```bash
# Format check
cargo fmt --all --check

# Lint (must pass with no warnings)
cargo clippy --workspace --all-targets --all-features -- -D warnings

# Run all tests
cargo test --workspace

# Run a single test by name
cargo test <test_name>

# Run the API server
cargo run --bin mem-api
```

### Environment Variables

```bash
# Required for embedding
export EMBED_API_URL=https://api.openai.com/v1/embeddings
export EMBED_API_KEY=sk-...

# Optional: server listen address
export MEMOS_LISTEN=0.0.0.0:8001

# Optional: Qdrant vector store
export QDRANT_URL=http://localhost:6334
export QDRANT_COLLECTION=memos

# Optional: persistent audit log
export AUDIT_LOG_PATH=./audit.jsonl
```

## Project Structure

This is a Rust workspace implementing a MemOS-compatible memory kernel:

- **mem-types**: DTOs, traits, lifecycle/audit types - core type definitions
- **mem-graph**: Graph store trait + in-memory implementation
- **mem-vec**: Vector store trait + in-memory + optional Qdrant backend  
- **mem-embed**: OpenAI-compatible embedding client + mock embedder
- **mem-cube**: `NaiveMemCube` orchestration (graph + vec + embedder)
- **mem-scheduler**: In-memory async add scheduler
- **mem-api**: Axum REST API server

## Code Style Guidelines

### General Conventions

- Use standard Rust 2021 edition
- Use `async`/`await` with Tokio runtime
- Use `thiserror` for custom error enums
- Use `async_trait` for async trait definitions
- Prefer `Arc<T>` for shared state, `Arc<dyn Trait>` for trait objects

### Naming Conventions

- **Types**: `CamelCase` (e.g., `ApiAddRequest`, `MemCubeError`)
- **Functions/Variables**: `snake_case` (e.g., `add_memories`, `memory_content`)
- **Constants**: `SCREAMING_SNAKE_CASE`
- **Traits**: `CamelCase` ending with `Trait` if not obvious (e.g., `GraphStore`)

### Imports

Group imports in this order:
1. Standard library (`std::`, `core::`)
2. External crates (alphabetical)
3. Local modules (`crate::`, `super::`)

```rust
use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use mem_types::{ApiAddRequest, MemoryResponse};
```

### Error Handling

Use `thiserror` for defining error types with context:

```rust
#[derive(Debug, thiserror::Error)]
pub enum MemCubeError {
    #[error("mem cube error: {0}")]
    Other(String),
    
    #[error("bad request: {0}")]
    BadRequest(String),
    
    #[error("not found: {0}")]
    NotFound(String),
    
    #[error("embedder: {0}")]
    Embedder(#[from] EmbedderError),
    
    #[error("graph: {0}")]
    Graph(#[from] GraphStoreError),
}
```

API handlers return JSON responses with `code`, `message`, and `data` fields:

```rust
Json(MemoryResponse {
    code: 200,
    message: "Success".to_string(),
    data: Some(result),
})
```

### Documentation

- Use doc comments (`///`) for public API items
- Document DTO fields with their purpose
- Include MemOS API compatibility notes where relevant

```rust
/// Add-memory request (MemOS APIADDRequest).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiAddRequest {
    /// User identifier for multi-tenant isolation
    pub user_id: String,
    /// Cube IDs the user can write to
    #[serde(default)]
    pub writable_cube_ids: Option<Vec<String>>,
}
```

### Struct Derives

Always derive the following for data types:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
```

Add `Default` where appropriate:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
```

### API Handler Patterns

Handlers follow this pattern:
```rust
async fn handle_add(
    State(state): State<Arc<AppState>>,
    Json(req): Json<ApiAddRequest>,
) -> Json<MemoryResponse> {
    match state.cube.add_memories(&req).await {
        Ok(res) => Json(res),
        Err(e) => Json(MemoryResponse {
            code: 500,
            message: e.to_string(),
            data: None,
        }),
    }
}
```

### Testing Patterns

Use `tokio::test` for async tests and `tower::ServiceExt` for HTTP testing:

```rust
#[tokio::test]
async fn add_sync_then_search() {
    let app = test_app();
    
    let add_body = json!({
        "user_id": "user1",
        "memory_content": "I like strawberries",
        "async_mode": "sync"
    });
    
    let req = Request::builder()
        .method("POST")
        .uri("/product/add")
        .header("content-type", "application/json")
        .body(Body::from(add_body.to_string()))
        .unwrap();
        
    let res = app.clone().oneshot(req).await.unwrap();
    // assertions...
}
```

### Where to Make Changes

- **DTO/Trait changes**: `mem-types` crate
- **Behavior orchestration**: `mem-cube` crate  
- **API protocol/status codes**: `mem-api` crate
- **New backend implementations**: Extend existing `GraphStore`/`VecStore` traits

### Testing Requirements

- Integration tests cover API behavior and cross-module flows
- Regression fixes must include tests
- Multi-tenant isolation and permission boundaries require explicit tests
- Use `tracing` for structured logging in handlers

### Pre-commit Checklist

Before submitting code, ensure:

```bash
cargo fmt --all --check    # Passes
cargo clippy --workspace --all-targets --all-features -- -D warnings  # No warnings
cargo test --workspace     # All tests pass
```

If changing API behavior, update `docs/api-reference.md`.  
If changing configuration, update `docs/configuration.md`.