# TUI Implementation Plan

## Purpose

This document records the agreed scope and implementation approach for the
initial AoC Suite TUI. It is an implementation plan, not a change to the settled
crate boundaries in `../design/architecture.md` or persistence policy in
`../design/storage.md`.

## Status

Last reviewed: 2026-07-31.

Overall status: **in progress**.

Implemented in the first slice:

- `aocsuite-tui` crate and explicit terminal lifecycle;
- pure tab and calendar state reduction;
- serialized background calendar and puzzle-description effects;
- cache-first puzzle descriptions and safe calendar refresh;
- released-year and day navigation;
- calendar completion rendering and explicit puzzle-description loading;
- foreground browser and exercise-editor handoff;
- deterministic reducer and `ratatui::TestBackend` render tests.

The Language and Config tab workflows remain pending.

## Initial scope

The initial TUI is a three-tab MVP:

- Calendar;
- Language;
- Config.

The TUI composes shared typed domain services directly. It must not invoke the
CLI or parse CLI output, and the existing CLI must remain functional.

Leaderboards are excluded. Solver execution, answer submission, Git, cleanup,
uninstall, and the remaining CLI workflows are deferred from this initial
release.

## Calendar tab

### Calendar and navigation

- Start on the configured year when that year has been released. Otherwise,
  select the latest released AoC year without changing configuration.
- Allow navigation from 2015 through the latest released AoC year.
- Render the parsed AoC calendar as a selectable grid while preserving its
  semantic colors.
- Show selected-year completion using AoC calendar stars:
  - earned stars out of the year's available total;
  - completed days.
- Use a split layout with the calendar and completion summary on the left and a
  puzzle-detail panel on the right.

Calendar navigation must not trigger puzzle-description http requests, puzzle descriptions can be loaded automatically if they are cached. Changing the
highlighted day clears the detail panel so that a previously loaded description
cannot be mistaken for the new selection.

### Puzzle descriptions

- Load the highlighted puzzle description only through an explicit user action.
- Use cached puzzle content when available and retrieve it otherwise.
- Display the resulting Markdown in the detail panel.
- Do not provide puzzle-description refresh in this phase.

### Actions

- Open the highlighted puzzle's Advent of Code page in the browser.
- Open the highlighted exercise in the configured editor by materializing:
  - puzzle Markdown;
  - the shared example file;
  - puzzle input;
  - the active language solution.
- Suspend and restore the terminal around foreground browser and editor
  processes.

### Calendar refresh

- Provide an explicit refresh action for the selected year's calendar only.
- Replace cached calendar content only after a successful response so a failed
  refresh cannot destroy valid cached content.

Typed calendar persistence and derived-star storage are not needed for this
phase. Completion is derived from the currently loaded semantic calendar.

## Language tab

- Maintain an in-session Rust or Python selection initialized from the
  configured default language.
- Do not persist a language selection from this tab. Persisted defaults are
  changed only from Config.
- Display and refresh package and library lists without introducing
  initialization side effects for query operations.
- Provide package add and remove dialogs.
- Provide library create/open and remove actions.
- Require confirmation before deleting a library.
- Provide template open/edit and reset actions.
- Require confirmation before resetting a template.
- Show running, success, and typed-error states for blocking operations.
- Suspend and restore the terminal around foreground editor processes.

Language jobs must be serialized for the runtime-root workspace as required by
the architecture and storage design.

## Config tab

Manage all existing configuration values:

- default year;
- default language;
- editor executable;
- run-history retention limit;
- session credential.

Validate values before saving. TUI year input is restricted to released years.

The session credential requires special handling:

- show only whether a session is configured;
- never place the existing credential in UI state;
- never render, log, snapshot, or include it in errors;
- use a masked field when setting or replacing it;
- require confirmation before removing it.

After a session change, future content effects must construct services using the
new credential state.

## Architecture

### Composition root

Add an `aocsuite-tui` workspace crate with a binary composition root that:

1. Resolves and bootstraps the runtime layout.
2. Ensures the workspace.
3. Loads configuration.
4. Initializes application state from effective settings and the latest
   released puzzle date.
5. Starts the terminal and event loop.
6. Restores the terminal on normal exit and errors.

### State and reducer

Use a pure state reducer for:

- active tab and focus;
- calendar year and highlighted puzzle;
- loaded puzzle details;
- language selection and lists;
- config form values;
- modal text input and confirmations;
- loading, success, and error notifications;
- user intents that request effects.

Rendering and state reduction must not perform filesystem, network, subprocess,
or other blocking work.

### Effects

Use a serialized background-effect runner for content and language operations.
Effects return semantic messages to the reducer rather than mutating rendered
state directly.

Construct the client and borrowing `ContentStore` within request-scoped effects.
This avoids self-referential ownership and ensures content requests use an
updated session credential after configuration changes.

Use a dedicated foreground-handoff path for launcher operations:

1. Suspend and restore the terminal to its normal mode.
2. Invoke the launcher operation.
3. Re-enter TUI terminal mode, including when the launcher reports an error.
4. Return the result to application state.

## Supporting domain changes

Keep supporting changes small and domain-owned:

- Add a content operation that returns puzzle Markdown for display while keeping
  cache and parsing policy inside storage.
- Add an explicit calendar refresh operation that safely replaces cached
  calendar content only after successful retrieval.
- Add a non-secret configuration query that reports whether a session is
  configured without returning the credential.

Do not add local progress queries, parsed leaderboard models, or generic
cross-domain orchestration services for this phase.

## Presentation

Build Ratatui views for:

- the tab strip;
- calendar grid and completion summary;
- scrollable Markdown puzzle details;
- package, library, and template management;
- configuration fields;
- modal text entry and confirmation dialogs;
- loading and error status;
- contextual keyboard help.

The layout must remain usable in both wide and narrow terminals.

## Implementation sequence

1. Add the `aocsuite-tui` crate, terminal lifecycle abstraction, application
   shell, and deterministic test terminal support.
2. Add the minimal content and configuration APIs required by the TUI.
3. Implement application state, messages, reducer, and effect requests.
4. Implement the serialized effect runner and foreground terminal handoff.
5. Implement the Calendar tab and its actions.
6. Implement the Language tab and destructive confirmations.
7. Implement the Config tab and credential-safe session flow.
8. Complete rendering, narrow-terminal behavior, keyboard help, and error
   handling.
9. Update active plan and user documentation to reflect implemented behavior.

## Tests

Add focused tests for changed behavior:

- storage tests for cache-first Markdown loading and safe calendar refresh
  failure behavior;
- config tests for session-status queries that do not expose credentials;
- reducer tests for tab navigation, released-year bounds, detail clearing,
  validation, confirmations, and effect dispatch;
- effect tests using fake command executors, deterministic service inputs, and
  fake terminal operations;
- `ratatui::TestBackend` rendering tests for all three tabs, loading and error
  states, narrow layouts, and masked session input;
- terminal lifecycle tests that verify restoration after normal exit and
  foreground-operation failures.

Normal tests must not contact Advent of Code or launch real external programs or
terminals.

## Verification

Run the broadest applicable workspace checks after implementation:

```text
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps
cargo check --workspace --locked
cargo test --workspace --locked
```
