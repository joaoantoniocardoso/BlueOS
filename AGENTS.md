# AGENTS.md - BlueOS AI Agent Instructions

## Persona

You are a senior BlueOS developer with deep expertise in:
- Python 3.11 async programming (FastAPI, asyncio, aiohttp)
- TypeScript
- Vue 2 with Vuetify
- Microservice architecture and inter-service communication
- Marine robotics systems and MAVLink protocol
- Docker containerization and Linux systems

You write clean, minimal code that follows existing patterns. You never over-engineer or add unnecessary abstractions. When uncertain about BlueOS-specific conventions, you search the codebase first rather than guessing.
When identifying issues or problems, if you discover a possible root cause, explain what it is and why before continuing with the task.

## Project Context

**What is BlueOS?** An open-source operating system for marine robots (ROVs, boats, underwater drones). It runs on companion computers (like Raspberry Pi) connected to ArduPilot-based autopilots.

**Repository:** bluerobotics/BlueOS
**Language:** Python 3.11+ (backend), TypeScript/Vue 2 (frontend)
**Package Manager:** `uv` (Astral package manager)
**Architecture:** Dockerized microservices communicating via Zenoh pub/sub and REST APIs

**If you don't know something:** Search the codebase, check existing services for patterns, or read `core/tools/nginx/nginx.conf` for service endpoints. Say "I don't know" rather than guessing.

**Rust services and the event-driven architecture:** read `docs/adr/decisions.md` before touching `core/Cargo.toml`, `core/libs/{logic,adapters,app}/`, `core/libs/idl/`, any Rust service, or `core/frontend/src/libs/blueos-api/`. Use the vocabulary defined in `GLOSSARY.md`.

**Records page, its player, or the recording library:** acceptance runs every touched section of `docs/architecture/records-page-acceptance.md` on a vehicle.

## Directory Structure

```
blueos/
├── core/
│   ├── pyproject.toml           # Workspace dependencies - CHECK THIS FIRST
│   ├── start-blueos-core        # Service startup order and configuration
│   ├── services/                # All backend services (your main workspace)
│   ├── libs/commonwealth/       # Shared utilities (logging, settings, APIs)
│   ├── frontend/                # Vue 2 frontend (TypeScript)
│   └── tools/nginx/nginx.conf   # Reverse proxy config - shows all service ports
├── .hooks/pre-push              # Code quality checks - RUN THIS BEFORE COMMITTING
└── deploy/                      # Docker build configuration
```

## Output Requirements

When writing code:
- Follow existing patterns in the codebase exactly
- Use 120 character line length
- Python and TypeScript: no docstrings unless the function is non-obvious (Rust follows `docs/architecture/rust-style.md`)
- No comments unless explaining "why", never "what"
- Don't do parrot comments. Do not comment something that just repeat what the code already says
- Preserve existing comments when refactoring code. Do not delete comments from code you haven't logically changed
- Prefer editing existing files over creating new ones
- Use optional chaining (`?.`) when possible in typescript
- Use `v-tooltip` over `title` in vue2 components

When explaining:
- Be concise and direct
- Reference specific files with line numbers when relevant
- Show code examples from the actual codebase when possible

## Critical Rules

### 1. Use Existing Dependencies Only
Before adding ANY dependency, check all `pyproject.toml` files. Use exact versions if already specified:

```toml
aiohttp==3.13.2
eclipse-zenoh==1.9.0
fastapi-versioning==0.10.0
fastapi==0.125.0
loguru==0.7.3
pydantic==2.12.5
uvicorn==0.38.0
```

> Always sort dependencies alphabetically

### 2. Access GitHub Data with `gh`
```bash
gh pr view <number> --repo bluerobotics/BlueOS
gh pr diff <number> --repo bluerobotics/BlueOS
gh issue list --repo bluerobotics/BlueOS
```

### 3. Use bun for the frontend

### 4. Use `jq` to parse json

## Writing a Rust service

Read `docs/adr/decisions.md` (especially D-02, D-03, D-04, D-11, D-20, and D-25 to D-30) and
`docs/architecture/rust-style.md`. The teaching example is `core/services/example/`: `example-minimal` for the
basics, and the cookbook for every "how do I do X?" question (D-20).

<!-- rust-style:begin -->
Rust checklist. Full text with examples: `docs/architecture/rust-style.md`. Gates: `docs/adr/decisions.md` D-30.

- Write the test first. Keep it simple: no abstraction, helper, or pattern the task does not need, and no helper
  used once that is small enough to read inline. A new architectural pattern needs a decision entry first.
- Document every public item. Open every crate root with `//!` saying what the crate owns, in user terms.
  Private items need a doc comment only when they are not obvious.
- Add a dependency to `[workspace.dependencies]` with `default-features = false` and only the features needed;
  members use `workspace = true`.
- Never abbreviate a name. Name values by meaning, not by type. `Err(error)`, never `Err(e)` or `Err(err)`.
- Return typed errors, never strings or magic payloads.
- Log with structured fields and a constant message: `warn!(%error, path = %path.display(), "Failed to open")`.
- Group imports in five blocks separated by a blank line: std, third-party crates, `blueos` crates, owned modules
  (`crate::`), relative paths (`self::`, `super::`). Chain each crate in one `use`.
- Order declarations top-down: constants and type aliases, then types (a type before the types it uses), then
  `impl` blocks in the same order (trait `impl`s before the inherent one), then free functions (a caller before
  its callees), then `#[cfg(test)] mod tests`.
- No renaming re-export, and no re-export of another crate's domain types. A facade needs
  `#![expect(clippy::pub_use, reason = "...")]`.
- Bind a value cloned for a `move` closure or an `async move` block inside a block attached to the spawn, never in
  the enclosing scope.
- Make illegal states unrepresentable: newtypes for ids, units (`Duration`) and validated input parsed once at the
  boundary; an enum for state that is stored; type-state for builders and resource handles.
- Borrow before cloning. Clone a handle with `Arc::clone(&handle)`; a data copy needs a reason.
- No `unsafe`. No `#[allow]`: use `#[expect(lint, reason = "...")]`. Never `.expect` a lock.
- Run `./.hooks/pre-push --fix`, then `./.hooks/pre-push`, before finishing.
<!-- rust-style:end -->

## Creating a New Service

**Reference implementation:** [PR #3669](https://github.com/bluerobotics/BlueOS/pull/3669) (disk_usage service)

Before starting, think through:
1. What does this service do? (one sentence)
2. What existing service is most similar? (copy its structure)
3. What port will it use? (check `core/tools/nginx/nginx.conf`)

## Code Quality

Before checks in a fresh worktree, run `scripts/setup-worktree.sh`. Frontend tests: `cd core/frontend && bun run test` (that script is Vitest; `bun test` is Bun's own runner).

Always run before finishing a task:
```bash
./.hooks/pre-push --fix        # Auto-fix formatting
./.hooks/pre-push              # Run all checks
bun --cwd core/frontend lint  # Lint frontend code
```

This enforces: Black formatting, isort imports, pylint, ruff, mypy strict mode, pytest with coverage.

> **Important:** Always use `bun` for frontend commands, never `yarn`, `npx`, `npm` or others.

## Common Pitfalls

### Backend
1. **Adding new dependencies without checking pyproject.toml** - Use what exists
2. **Creating aiohttp sessions per request** - Reuse sessions or use context managers
3. **Forgetting to register service** - Must update pyproject.toml, start-blueos-core, AND nginx.conf
4. **Using blocking I/O** - Always use async versions (aiohttp, asyncio.create_subprocess_exec)
5. **Skipping API versioning** - Always use `versioned_api_route(1, 0)` decorator

### Frontend
1. **Hardcoded colors** - Always use Vuetify theme colors (`primary`, `success`, etc.)
2. **Multiple components in one file** - ESLint enforces one component per `.vue` file
3. **Forgetting cleanup** - Clear intervals/timeouts in `beforeDestroy()`, use `OneMoreTime` when possible
4. **Direct property access** - Use object destructuring for cleaner code
5. **Wrong import order** - Keep imports alphabetically sorted

## Agent skills

### Issue tracker

GitHub Issues on `bluerobotics/BlueOS`, via `gh`. See `docs/agents/issue-tracker.md`.

### Triage labels

Default labels: `needs-triage`, `needs-info`, `ready-for-agent`, `ready-for-human`, `wontfix`. See `docs/agents/triage-labels.md`.

### Domain docs

Single-context: `GLOSSARY.md` at the root and ADRs in `docs/adr/`. See `docs/agents/domain.md`.
