# mem-kernel-rs

Rust implementation of [MemOS](https://github.com/MemTensor/MemOS)-compatible memory API: unified add/search over graph + vector storage for LLM/agent memory.

## Layout

- **mem-types** – DTOs and traits (`ApiAddRequest`, `ApiSearchRequest`, `MemoryResponse`, `SearchResponse`, `GraphStore`, `VecStore`, `Embedder`, `MemCube`)
- **mem-graph** – `GraphStore` + in-memory implementation (nodes + embedding KNN)
- **mem-vec** – `VecStore` + in-memory and optional [Qdrant](https://qdrant.tech/) (`--features qdrant`)
- **mem-embed** – OpenAI-compatible embedding HTTP client
- **mem-cube** – `NaiveMemCube`: add/search flow using graph, vector store, and embedder
- **mem-api** – REST server (Axum): `POST /product/add`, `POST /product/search`

## Run the API

Default: in-memory graph and vector store. Set embedding API for add/search:

```bash
export EMBED_API_URL=https://api.openai.com/v1/embeddings   # or any OpenAI-compatible URL
export EMBED_API_KEY=sk-...
export MEMOS_LISTEN=0.0.0.0:8001   # optional, default 8001

cargo run --bin mem-api
```

To use **Qdrant** as the vector backend (persistent), set `QDRANT_URL` (and optionally `QDRANT_COLLECTION`). The binary will choose the store at startup:

```bash
export QDRANT_URL=http://localhost:6334
cargo run --bin mem-api
```

**Multi-tenant / namespace:** All storage is isolated by `user_id` and `mem_cube_id` (or `writable_cube_ids` / `readable_cube_ids`). Use the same identifiers in add and search for a given user/cube.

Then:

- **Add:** `POST http://localhost:8001/product/add` with JSON body like MemOS, e.g. `{"user_id":"u1","mem_cube_id":"u1","messages":[{"role":"user","content":"I like strawberry"}],"async_mode":"sync"}`.
- **Search:** `POST http://localhost:8001/product/search` with `{"query":"What do I like","user_id":"u1","mem_cube_id":"u1","top_k":10}`.

Request/response shapes match MemOS so existing clients can target this service.

## License

Apache-2.0.
