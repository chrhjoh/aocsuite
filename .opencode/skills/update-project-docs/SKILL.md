---
name: update-project-docs
description: Update AoC Suite design or plan documentation when a code change makes it stale.
compatibility: opencode
metadata:
  project: aocsuite
  workflow: documentation
---

# Update project documentation

Use this skill only when the requested change alters documented behavior,
architecture, or active-plan status.

## Route changes

- use the `AGENTS.md` as source of truth for routing of documentation

## Rules

- Update the authoritative document rather than duplicating the decision.
- Stable documents describe accepted design, not task progress.
- Active plans describe status and sequencing, not full design.
- Do not rewrite design documents merely to justify a conflicting
  implementation.
- Do not archive or delete an active plan without explicit permission.
- Do not mark an item complete unless its acceptance criteria are satisfied and
  relevant checks were run.
