description: Create a focused Conventional Commit

Inspect the current status and diff. Stage only the changes that form one
coherent commit, then run these verification checks:
- cargo fmt --all -- --check
- cargo check --workspace --locked
- cargo test --workspace --locked

Create a Conventional Commit using this context:

$ARGUMENTS

Do not amend, discard changes, include unrelated files, or commit if the staged
diff contains credentials or unresolved failures.

Report the commit message, hash, included files, checks run, and remaining
changes.
