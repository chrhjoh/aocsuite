# TUI Implementation Plan

## Purpose

This document records the agreed scope and implementation approach for the
initial AoC Suite TUI. It is an implementation plan, not a change to the settled
crate boundaries in `../design/architecture.md` or persistence policy in
`../design/storage.md`.

## Status

Last reviewed: 2026-08-01.

Overall status: **in progress**.

Implemented in the first slice:

- `aocsuite-tui` crate and explicit terminal lifecycle;
- pure tab and calendar state reduction;
- serialized background calendar and puzzle-description effects;
- cache-first puzzle descriptions and safe calendar refresh;
- released-year and visual calendar-puzzle navigation;
- calendar completion rendering, cached Markdown preview, and explicit redownload;
- foreground browser and exercise-editor handoff;
- in-session language selection, package and library management, template
  editing and reset, and destructive confirmations;
- independently saved Config fields and credential-safe session set, replace,
  status, and confirmed removal flows;
- deterministic reducer and `ratatui::TestBackend` render tests.

Final presentation review and user documentation remain pending.

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
- Navigate puzzle links in their parsed visual order: rows from top to bottom
  and cells from left to right. Skip rows without puzzle links, treat repeated
  cells from a multi-line puzzle link as one target, and do not wrap at the
  first or last puzzle.
- Select the first puzzle in visual order after the initial or a year-changing
  calendar load. A calendar refresh preserves the selected puzzle when it is
  still present and otherwise selects the first puzzle.
- Puzzle navigation moves the selection without moving the calendar viewport.
  Keep Ctrl+arrow manual scrolling available when the terminal is too short or
  narrow to show the full calendar at once.
- Highlight only the final visual row of the selected puzzle link so a
  multi-line entry has one selection line.
- Show selected-year completion using AoC calendar stars:
  - earned stars out of the year's available total;
  - completed days.
- Use a split layout with the calendar and completion summary on the left and a
  puzzle-detail panel on the right.

After the calendar establishes its initial selection, calendar navigation
automatically loads valid cached puzzle Markdown without triggering
puzzle-description HTTP requests. Changing the highlighted puzzle clears the
detail panel body while that cache lookup runs, retaining only the selected day
in the panel title, so that a previously loaded description cannot be mistaken
for the new selection. Do not show loading text or the download prompt until the
cache lookup confirms that Markdown is absent.

While the initial calendar load is pending, keep both the calendar body and the
puzzle-detail body blank. Continue to show confirmed calendar failures and their
retry instruction.

### Puzzle descriptions

- When cached Markdown is absent, prompt the user to press `d` to download the
  highlighted puzzle description.
- The download action always retrieves the puzzle page and replaces its cached
  HTML and Markdown, including when a preview is already loaded. This allows an
  updated description to be retrieved after completing part one.
- Keep an existing preview visible while it is redownloaded. A failed download
  preserves that preview and reports the error; without a preview, display the
  error in the detail panel.
- Ignore repeated download requests for the same puzzle while one is pending.
- Downloads continue after navigation and update their requested puzzle's
  cache, but stale results do not update the selected puzzle's UI.
- Display the resulting Markdown in the detail panel.
- Scroll loaded Markdown by wrapped visual line with PageUp and PageDown. Show a
  trackless Unicode thumb on the right edge when the wrapped content exceeds
  the visible detail pane.
- Do not provide a separate puzzle-description refresh action; `d` always
  downloads and therefore also refreshes an existing preview.

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
- Use that selection for all TUI language operations, including Calendar-tab
  exercise materialization.
- Do not expose or persist default-language configuration in the TUI.
- Display and refresh package and library lists without introducing
  initialization side effects for query operations.
- Provide package add and remove dialogs.
- Provide library create/open and remove actions.
- Require confirmation before deleting a library.
- Provide template open/edit and reset actions.
- Require confirmation before resetting a template.
- Show continuing package and library operations in the affected pane border.
  Keep template preparation silent, make routine success implicit in refreshed
  content or editor handoff, and show typed errors in a dismissible dialog.
- Suspend and restore the terminal around foreground editor processes.

Language jobs must be serialized for the runtime-root workspace as required by
the architecture and storage design.

## Config tab

Manage the configuration values exposed by the TUI:

- default year;
- editor executable;
- run-history retention limit;
- session credential.

Save each field independently. Validate values before saving, make routine
success implicit in the refreshed field value, and show failures in a
dismissible dialog. TUI year input is restricted to released years and affects
future startup without navigating the current Calendar tab. When no year is
persisted, display the latest released year; blank input removes the override.

Preserve exact editor executable text after trimming its outer whitespace, and
use blank input to return to the configured fallback. Run-history retention must
remain a positive integer; blank input returns to its effective default.
Use `x` to reset any selected non-secret field; on the session field, `x`
initiates the confirmed removal flow.

The session credential requires special handling:

- show only whether a session is configured;
- never place the existing credential in UI state;
- never render, log, snapshot, or include it in errors;
- use a masked field when setting or replacing it;
- require nonempty trimmed input without contacting Advent of Code;
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
- local loading state and non-routine result, warning, and error notifications;
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

- Add content operations that read an existing managed Markdown preview without
  fallback and that explicitly redownload and safely replace puzzle HTML and
  Markdown, while keeping cache and parsing policy inside storage.
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
- a hidden, tab-specific keyboard reference opened with `?` instead of
  persistent key lists in the footer.

Reserve global footer status for errors, warnings, and rejected actions that do
not require acknowledgment. Routine lifecycle progress and success
acknowledgments remain implicit or use local pane indicators instead of footer
status. Meaningful future results such as answers, solver timings, and
submission outcomes use dismissible dialogs rather than transient footer text.

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

- storage tests for cached-only Markdown previews, safe puzzle redownload, and
  safe calendar refresh failure behavior;
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
