# CLAUDE.md — Rust Market Data Viewer

## Source of truth
- Read `plan.txt` first and follow it.
- Acceptance criteria in `plan.txt` is the definition of done.

## Non-negotiables
- Never expose `DATABENTO_API_KEY` to the frontend.
- Do not commit secrets. Use `.env` locally + provide `.env.example`.
- Keep the monorepo structure:
  - Rust workspace at repo root
  - `crates/shared` for request/response types shared by backend
  - `crates/backend` for Axum server + Databento integration
  - `ui/` for React+Vite+TS frontend

## Defaults / Ports
- Backend listens on `http://127.0.0.1:3001`
- Frontend (Vite) on `http://127.0.0.1:5173`
- Frontend calls backend via `/api/*` (configure Vite proxy in dev)

## Databento integration policy (important)
- Default to MOCK mode if `DATABENTO_API_KEY` is not set.
- The build must not require a Databento account/key to run end-to-end.
- If `DATABENTO_API_KEY` is present, switch to LIVE mode automatically.

## API Credit Conservation (CRITICAL)
- **DO NOT** run tests or make API calls that consume DataBento credits unless explicitly requested by the user.
- **DO NOT** use the real DataBento API for development testing - use MOCK mode instead.
- When implementing DataBento integration, write code without testing against the real API.
- Only test with real API when the user explicitly asks to verify the integration.
- We have limited credits - be conservative and always prefer MOCK mode for development.

## How to run (dev)
### Backend
- Requires env var: `DATABENTO_API_KEY`
- Run:
  - `export DATABENTO_API_KEY="..."`
  - `cargo run -p backend`

### Frontend
- Run:
  - `cd ui`
  - `npm install`
  - `npm run dev`

## Verification loop (must do after meaningful changes)
### Rust
- `cargo fmt --all`
- `cargo clippy --workspace --all-targets -D warnings`
- `cargo test --workspace`

### UI
- `cd ui && npm run build`
- If lint/typecheck exists, run them too.

### Smoke checks
- `curl -s http://127.0.0.1:3001/api/health`
- Verify `/api/historical` returns valid JSON for trades and ohlcv.
- Verify WS `/ws/live` connects from the UI and streams messages.

## Implementation guidance
- Prefer correctness + debuggability over clever abstractions.
- Keep Databento integration isolated (e.g. `databento_service.rs`).
- When uncertain about Databento crate APIs, compile frequently and adapt to what exists.
- Add tracing logs for major actions: startup, requests, subscription, stream events.
- Keep changes incremental and keep the project building at each step.

## Workflow preference
- Start in Plan mode. Propose a plan with milestones.
- After plan is approved, implement in small steps and verify each step.