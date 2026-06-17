# HelpDocApp Repository Structure

## Folder Structure

```
HelpDocApp/
├── backend/          # Rust server (Actix-web)
├── frontend/         # Next.js React application
├── .git/             # Version control
└── help-docs-ai-masterplan.md  # Project planning document
```

### Backend (`backend/`)
- **src/** - Main Rust source code
- **migrations/** - Database migrations (Diesel)
- **python_services/** - Python embedding service
- **tests/** - Integration/unit tests
- **Cargo.toml** - Rust dependencies and metadata
- **Cargo.lock** - Locked dependency versions
- **diesel.toml** - ORM configuration
- **log4rs.yaml** - Logging configuration
- **db_schema.rs** - Database schema definition

### Frontend (`frontend/`)
- **src/** - TypeScript/React source code
  - `components/` - React components
  - `pages/` - Next.js page routes
  - `styles/` - CSS/styling
- **public/** - Static assets
- **package.json** - Node dependencies
- **tsconfig.json** - TypeScript configuration
- **next.config.mjs** - Next.js configuration
- **tailwind.config.ts** - Tailwind CSS configuration

---

## Main Entry Points

**Backend:**
- `backend/src/main.rs` - Starts Actix-web HTTP server on `127.0.0.1:3000`
  - Initializes services: ChatServer, EmbeddingService, AIService, SearchService, MetadataGenerator
  - Configures CORS, logging, and database pool
  - Registers routes from `routes` module

**Frontend:**
- `frontend/src/pages/` - Next.js page routes (accessed on `http://localhost:3000`)
- Entry point runs with `npm run dev`

---

## Config and Setup Files

**Backend:**
- `Cargo.toml` - Project metadata, dependencies (Actix, Diesel, Tokio, etc.)
- `diesel.toml` - Diesel ORM settings (migrations directory, database URL)
- `log4rs.yaml` - Structured logging configuration

**Frontend:**
- `package.json` - Dependencies (Next.js, TypeScript, Tailwind)
- `tsconfig.json` - TypeScript compiler options
- `next.config.mjs` - Next.js server config
- `.eslintrc.json` - ESLint rules

**Root:**
- `.gitignore` - Git exclusions
- `help-docs-ai-masterplan.md` - Project scope, features, technical stack

---

## Tests and Docs

**Tests:**
- `backend/tests/` - Test suite for backend

**Documentation:**
- `help-docs-ai-masterplan.md` - High-level project overview and objectives
- `frontend/README.md` - Standard Next.js setup instructions

**Migrations:**
- `backend/migrations/` - Diesel SQL migrations tracking schema evolution
  - Initial schema (Sept 2024)
  - Embeddings table, article chunks, metadata tables
  - Column updates (timestamps, keywords, bulletpoints)

---

## Core Code Areas

**Backend Architecture:**
- `src/main.rs` - Server entry point and service initialization
- `src/routes/` - HTTP endpoint definitions
- `src/services/` - Business logic
  - `chat/` - WebSocket chat server
  - `search.rs` - Search functionality
  - `ai_service.rs` - LLM integration
  - `embedding_service.rs` - Vector embeddings
  - `metadata_generator.rs` - Content metadata extraction
  - `data_processor.rs` - Data ingestion pipeline
- `src/models/` - Data structures
- `src/db/` - Database connection and pool management
- `src/graphql/` - GraphQL schema/resolvers
- `src/utils/` - Helper functions
- `src/errors.rs` - Error handling types

**Frontend Structure:**
- `src/pages/` - Next.js routes
- `src/components/` - Reusable React components
- `src/styles/` - Global styles

**Supporting:**
- `backend/python_services/embedding_service.py` - Python service for embeddings (commented out in main.rs)
- `backend/db_schema.rs` - Diesel schema definition
- `backend/src/schema.graphql` - GraphQL schema definition

---

## Notes for Deeper Review

1. **Python Service Status**: Python embedding service exists (`python_services/embedding_service.py`) but is currently commented out in `main.rs` (lines 35-40, 97). Verify if this is intentional or needs to be re-enabled.

2. **Service Integration**: Main.rs initializes multiple services (ChatServer, EmbeddingService, AIService, SearchService, MetadataGenerator). Check route handlers to see which are actively used.

3. **GraphQL Setup**: Schema file exists (`schema.graphql`) and Rust models reference GraphQL, but schema contents are minimal (260 bytes). Verify implementation completeness.

4. **Database Layer**: Uses Diesel ORM with PostgreSQL. Multiple migrations suggest evolving schema. Check migration order and rollback safety.

5. **Frontend Pages**: Pages directory exists but README shows generic Next.js template. Check actual page implementations to understand UI structure.

6. **API Routes**: Frontend has `pages/api/` structure typical for Next.js. Verify whether these proxy to backend or serve independently.
