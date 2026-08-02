# Post-MVP TUI Workflow Parity Plan

## Status

Last reviewed: 2026-08-02.

Overall status: **active**.

This plan succeeds `tui-implementation.md` and tracks the remaining deliberate
TUI workflow-parity slices. Existing CLI workflows remain supported.

## Sequence

1. **Solver execution - implemented.** Run the selected Calendar puzzle with the
   current in-session language, a selected part, and AoC or shared-example input;
   display structured output and persist timings in the serialized worker.
2. **Answer submission - implemented.** Calendar asks for part and answer, with
   that explicit entry serving as confirmation before immediate submission. An
   eligible successful single-part AoC-input run result instead retains its
   puzzle, part, and exact answer and uses a cancel-by-default confirmation.
   Both paths render semantic outcomes and refresh currently visible affected
   content after correct answers.
3. **Git through lazygit - implemented.** Add frontend-owned terminal handoff
   to lazygit for the workspace without wrapping or parsing its output.
4. **Releases - unimplemented.** Add TUI artifacts to the existing release
   process after workflow parity and cross-platform readiness are established.

## Exclusions

TUI leaderboards are explicitly excluded. Do not remove or change the existing
CLI leaderboard behavior.
