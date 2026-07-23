---
description: Review the current diff against the task and project design
agent: reviewer
subtask: true
---

If git diff or git diff --cached shows no changes, report "No changes to review" and stop.

Otherwise, review the current Git diff.

use skill verify-rust-change $ARGUMENTS

Pay particular attention to deviations from these criteria and from the
authoritative project documents.
