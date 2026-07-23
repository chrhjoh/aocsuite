---
name: verify-rust-change
description: Verify a completed Rust behavior change in AoC Suite before reporting completion.
compatibility: opencode
metadata:
  project: aocsuite
  workflow: verification
---

# Verify a Rust change

Use this skill after implementing or materially changing Rust behavior.

## Procedure

1. Read the verification section of `AGENTS.md`.
2. Inspect the diff for unrelated changes.
3. Identify the public or domain behavior that changed.
4. Check whether focused tests cover that behavior.
5. Run focused checks where they improve diagnosis.
6. Run the broadest applicable repository checks.
7. Do not weaken checks, remove tests, or hide failures.
8. Check whether authoritative documentation became stale. use skill update-project-docs

## Report

Return:

- behavior verified;
- tests added or updated;
- commands run and results;
- unrelated existing failures;
- documentation updated;
- remaining uncertainty.
