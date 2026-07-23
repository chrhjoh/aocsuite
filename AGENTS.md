# AoC Suite Agent Instructions

## Scope

Implement only the requested behavior and the smallest supporting changes.

Do not perform unrelated architecture migrations, refactors, cleanup, dependency
changes, or roadmap work. Do not advance items from design or implementation
plans unless they are part of the requested task.

If the requested change conflicts with a documented decision, state the conflict
before editing. When documents conflict, do not choose silently.

## Sources of truth

Use this order when interpreting the repository:

1. The user's instructions for the current task.
2. This file for repository-wide working constraints.
3. Domain design documents for settled architecture and behavior.
4. Active plans for current status, sequencing, and unfinished work.
5. Existing implementation and tests as evidence of current behavior.

An active plan does not authorize unrelated work. Existing behavior is not
automatically authoritative when a design document records an approved change.

Read only the domain documents relevant to the task:

- `docs/design/architecture.md` for crate ownership and dependency direction.
- `docs/design/storage.md` for runtime layout, persistence, migration, and cleanup.
- `docs/ci.md` for CI and release automation.
- `docs/plans/pre-tui-refactor.md` for current migration status and sequencing.

## Product constraints

- Keep `aocsuite-cli` functional throughout changes.
- CLI and TUI use the same typed domain services.
- Do not drive one frontend through another or parse frontend output.
- Frontends may differ in presentation, confirmation, terminal handoff, and job
  scheduling.
- Blocking filesystem, network, and subprocess work must not run in Ratatui
  update or render code.
- The application currently assumes one AoC Suite process per runtime root. Do
  not introduce normal cross-process coordination unless explicitly requested.

## Architecture

- Frontends own argument or event handling, prompts, rendering, terminal
  lifecycle, confirmation, and frontend-specific scheduling.
- Domain crates receive individual configuration values, paths, clients, and
  executors explicitly.
- Domain crates do not prompt, print, render, or discover global runtime state.
- Policy belongs to its owning domain.
- Do not create a generic orchestration, operations, application-services, or
  similarly cross-cutting crate to hold domain policy.
- A thin frontend composition root may construct dependencies and coordinate
  domain services.
- Do not introduce dependencies that reverse the documented crate dependency
  direction.

## Safety and compatibility

- Never print, log, snapshot, or otherwise expose session credentials.
- Never contact live Advent of Code services in tests.
- Destructive services receive already-confirmed typed scopes and do not prompt.
- Do not overwrite, delete, or claim ownership of unknown user files.
- Do not change persisted formats, migrations, cleanup semantics, CLI command
  shapes, configuration precedence, or confirmation behavior unless explicitly
  requested.
- Read-only operations must not create or modify persistent state unless
  initialization is an explicit part of the operation.
- Do not commit unless explicitly requested.

## Errors

- Preserve operation context, relevant paths or commands, and source errors.
- Do not replace typed library errors with generic strings.
- Libraries return semantic data and typed errors; frontends render
  user-facing prose.
- Errors and diagnostics must not expose credentials or sensitive HTTP headers.

## Tests and verification

Add or update tests for the changed behavior.
Include focused unit tests for internal logic and tests that verify public-facing functions behave as expected.
Do not broadly restructure the test suite, introduce a new test framework, or expand coverage into unrelated areas unless explicitly requested.

Use explicit temporary roots and deterministic clock, environment, process, and
HTTP seams. Normal tests must not invoke real Git, Cargo, Python, pip, editors,
browsers, terminals, or Advent of Code requests.

Run the broadest applicable checks:

- `cargo fmt --all -- --check`
- `cargo check --workspace --locked`
- `cargo test --workspace --locked`
