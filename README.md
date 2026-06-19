# Rag Engine RS

An AI-powered help-documentation backend written in Rust. It ingests articles
from a HelpScout-style API, converts them to Markdown, chunks and embeds them
into PostgreSQL (via `pgvector`), and serves **hybrid semantic + keyword
retrieval** alongside a **streaming LLM chat** interface over WebSockets —
with all inference running locally through [Ollama](https://ollama.com).

**Backend only.** No UI is included; this project is an AI infrastructure
service, not a full-stack app.

---

## Architecture

```
HelpScout-style REST API
  │
  ▼
api_client  ──── paginated fetch, 30 s timeout
  │
  ▼
HTML → Markdown  (html2md + custom scraper rules)
  │
  ▼
Diesel / PostgreSQL  ─── store canonical articles
  │
  ├── chunk (500 chars) ──► Python embedding service ──► pgvector
  │                          localhost:8080/embed
  └── metadata pipeline ──► Ollama (llama3.1) ──► keywords, summaries
        (bounded concurrency: Semaphore + buffer_unordered)
  │
  ▼
Two-stage hybrid retrieval
  Stage 1: semantic vector search   (pgvector cosine similarity)
  Stage 2: keyword re-rank          (scoped to stage-1 candidate set)
  Score fusion → ranked article list
  │
  ▼
Actix actor model ── WebSocket chat
  ChatServer ── per-session ChatSession actors
  RAG grounding (retrieved articles → prompt)
  Token streaming over WS (Ollama stream API)
```

---

## Engineering highlights

**Hybrid two-stage retrieval** (`src/services/search/two_stage_retrieval.rs`)
Semantic vector search narrows the candidate set; keyword re-rank then scores
within that set and fuses the two signals. This beats pure vector search on
exact-term queries without sacrificing recall on paraphrased ones.

**Actix actor model for streaming chat** (`src/services/chat/`)
Each WebSocket connection becomes a `ChatSession` actor; a central `ChatServer`
actor routes messages and manages session state. Ollama's streaming API is
consumed token-by-token and forwarded over the WS connection in real time —
the client sees words appear as they're generated.

**Custom round-robin Ollama load balancer** (`src/utils/ollama_load_balancer.rs`)
A round-robin balancer backed by a Rayon thread pool distributes inference
requests across multiple local Ollama instances. Included but currently shelved
pending multi-GPU hardware; the interface is stable and ready to re-wire.

**Bounded-concurrency metadata pipeline** (`src/services/metadata_generator/`)
A `tokio::sync::Semaphore` + `buffer_unordered` pipeline caps in-flight Ollama
calls during bulk metadata generation, with structured success/partial/failure
accounting and failed-ID persistence for retry.

---

## Tech stack

| Layer | Technology |
|---|---|
| Language | Rust (2021 edition) |
| HTTP / WebSocket | Actix-web 4, Actix actors |
| Database | PostgreSQL + `pgvector` extension |
| ORM / migrations | Diesel 2 |
| LLM inference | Ollama (local) via vendored `ollama-rs` fork |
| Embeddings | Python microservice (`python_services/`) |
| Async runtime | Tokio |
| HTML → Markdown | html2md + scraper |

---

## Quickstart

### Prerequisites

- **Rust** 1.78+ (`rustup update stable`)
- **PostgreSQL** with the [`pgvector`](https://github.com/pgvector/pgvector) extension installed
- **Diesel CLI** — `cargo install diesel_cli --no-default-features --features postgres`
- **Ollama** running locally with a chat model pulled, e.g.:
  ```
  ollama pull llama3.1
  ```
- **Python 3.9+** for the embedding service

> **macOS / Homebrew note:** if Postgres was installed via Homebrew you may
> need to export the library path before building:
> ```
> export LIBRARY_PATH="/opt/homebrew/opt/libpq/lib:$LIBRARY_PATH"
> ```

### 1. Clone and configure

```bash
git clone <repo-url>
cd rag-engine-rs
cp .env.example .env
# Edit .env — at minimum set DATABASE_URL, API_KEY, API_BASE_URL
```

### 2. Set up the database

```bash
diesel migration run
```

### 3. Start the Python embedding service

```bash
cd python_services
pip install -r requirements.txt
python embedding_service.py   # listens on localhost:8080
```

### 4. Run the backend

```bash
cargo run --release
# Server starts at http://127.0.0.1:3000
```

### 5. API endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/` | Health check |
| POST | `/health` | Embedding service health probe |
| GET | `/ws` | WebSocket chat (upgrade here) |
| POST | `/search` | Hybrid article search |
| POST | `/parse` | Trigger article sync from source API |
| POST | `/metadata-generation` | Run metadata pipeline on articles |
| POST | `/embed` | Generate and store embeddings |
| GET | `/failed-embedding-articles` | List articles that failed embedding |
| POST | `/reembed-all` | Re-embed all articles from scratch |
| GET | `/job-status` | Current job queue status |

### 6. WebSocket chat

Connect to `ws://127.0.0.1:3000/ws`. Send JSON:

```json
{ "message": "How do I reset my password?" }
```

The server streams back tokens as they arrive from Ollama, grounded in
articles retrieved by the hybrid search pipeline.

---

## Vendored `ollama-rs` fork

This project uses a fork of
[`ollama-rs`](https://github.com/pepperoni21/ollama-rs) vendored at
`vendor/ollama-rs`. The upstream crate is at 0.3.5; we stay on 0.2.0 because
the history API changed in a way that is incompatible with the per-session
Actix actor model — upstream now requires an externally managed
`Arc<Mutex<MessagesHistory>>`, but the actor model owns history per-session
internally via `new_default_with_history`. One borrow-checker fix was applied
on top of the upstream 0.2.0 tag. See [`vendor/ollama-rs/VENDORED.md`](vendor/ollama-rs/VENDORED.md)
for the full diff description.

---

## License

MIT — see [LICENSE](LICENSE).
