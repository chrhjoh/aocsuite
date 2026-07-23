# Continuous Integration and Releases

## Status and purpose

This document defines the intended GitHub Actions and release policy.

It separates stable automation decisions from rollout status. Current completion
status is tracked in `plans/pre-tui-refactor.md`.

## Goals

- Verify pull requests and default-branch changes.
- Exercise supported operating systems with deterministic tests.
- Publish reproducible CLI binaries.
- Add TUI binaries to the same product release after TUI parity.
- Keep CI independent of credentials, live services, user applications, and
  developer-local state.

CI must never:

- use a real AoC session;
- contact Advent of Code;
- submit answers;
- launch editors, browsers, or a real terminal UI;
- depend on a developer runtime root;
- run untrusted pull-request code with write permissions.

Tests use explicit temporary roots and deterministic HTTP, process, clock,
environment, filesystem, and terminal seams.

## Rollout stages

Stages describe sequencing, not standing permission to perform unrelated CI
work.

### Stage 1: Baseline CI

The baseline workflow runs on pull requests and default-branch pushes:

```text
cargo check --workspace --locked
cargo test --workspace --locked
cargo run -p aocsuite-cli --locked -- --help
```

Initial runner:

- Ubuntu;
- stable Rust;
- explicit temporary `AOCSUITE_DATA_DIR`;
- read-only repository permissions;
- no secrets.

Use concurrency groups to cancel superseded runs for the same branch.

Formatting, strict Clippy, and rustdoc are not required until their existing
baselines pass.

### Stage 2: Deterministic cross-platform matrix

After normal tests no longer invoke real process, environment, HTTP, terminal,
or developer-local state, run stable Rust on:

- Ubuntu x86-64;
- Windows x86-64;
- macOS ARM64.

Each entry runs:

```text
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo run -p aocsuite-cli --locked -- --help
```

Install a supported Python version with `actions/setup-python` where tests
inspect Python-project behavior. Normal required tests must not create real
virtual environments or invoke pip.

Optional real Cargo or Python smoke tests belong in a separate non-required
workflow.

After `aocsuite-tui` exists, compile and test it noninteractively with
`ratatui::TestBackend` and fake terminal operations. Do not launch a real TUI.

### Stage 3: Quality gates

After their baselines pass, add separate required jobs for:

```text
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps
```

Keep quality jobs separate from the OS matrix to classify failures clearly and
avoid redundant work.

## CI workflow shape

`.github/workflows/ci.yml` should eventually contain:

### `quality`

- Ubuntu stable Rust.
- Formatting, Clippy, and rustdoc.
- No runtime secrets.

### `test`

- Ubuntu, Windows, and macOS matrix.
- Locked workspace check and tests.
- Explicit Python setup where needed.
- CLI help smoke test.
- TUI unit and render tests after the crate exists.

### Boundary assertions

Prefer Rust tests in the owning crate over fragile shell inspection.

Useful assertions include:

- shared crates compile without frontend-only dependencies;
- no normal test requires an AoC session;
- generated roots remain inside temporary directories;
- external processes and network calls use fakes in normal coverage.

A dedicated `feature-boundaries` job is optional; create it only when it adds
clear signal beyond ordinary tests.

## Actions and permissions

Preferred actions:

- `actions/checkout`;
- `dtolnay/rust-toolchain`;
- `Swatinem/rust-cache`;
- `actions/setup-python`.

Pin third-party actions to immutable commit SHAs. Configure Dependabot to update
GitHub Actions references.

Build and test jobs use read-only permissions. Only the release upload job
receives `contents: write`.

## Release versioning

Use one product version across CLI, future TUI, and internal workspace crates
while they are released together.

Release tags use:

```text
vMAJOR.MINOR.PATCH
```

The tag must match synchronized workspace package versions.

Initial binary releases do not publish crates to crates.io. Publishing
path-connected internal crates requires a separate policy and release order.

## Release preparation

1. Update workspace package versions consistently.
2. Update `Cargo.lock`.
3. Record user-visible changes in release notes or a changelog.
4. Merge through required CI.
5. Create an annotated version tag.

## Release workflow

`.github/workflows/release.yml` is triggered by version tags and may support
`workflow_dispatch` for dry runs.

It must:

1. validate the tag against the workspace product version;
2. run or require the complete CI suite;
3. build native release binaries with `--locked`;
4. execute each native binary with `--help`;
5. package binaries with README and license files;
6. generate SHA-256 checksums;
7. create a GitHub Release and upload archives and checksums.

Initial CLI targets:

```text
x86_64-unknown-linux-gnu
x86_64-pc-windows-msvc
aarch64-apple-darwin
```

Prefer native GitHub runners.

Suggested artifact names:

```text
aocsuite-cli-v0.4.0-x86_64-unknown-linux-gnu.tar.gz
aocsuite-cli-v0.4.0-x86_64-pc-windows-msvc.zip
aocsuite-cli-v0.4.0-aarch64-apple-darwin.tar.gz
```

After TUI parity, prefer one product archive containing both binaries when they
share versioning and platform support. Use separate archives only when
installation or support differs.

## Additional automation

### Dependencies and licenses

Add a weekly and dependency-changing pull-request workflow running:

```text
cargo deny check advisories bans licenses sources
```

Prefer `cargo-deny` over overlapping audit and custom license scripts.

Configure Dependabot for Cargo and GitHub Actions.

### Coverage

Coverage is optional and Linux-only initially:

```text
cargo llvm-cov --workspace --lcov
```

Coverage trends may be uploaded, but a global percentage gate should not block
merges. Prefer targeted behavioral requirements.

### Parser fuzzing

AoC HTML is external input. Scheduled or manual fuzz targets for puzzle,
calendar, and submission parsers may be useful after parser APIs are pure and
fixture coverage is established.

### Supply chain and static analysis

GitHub dependency review is useful on pull requests. CodeQL is secondary to
Clippy, `cargo-deny`, and focused behavioral tests.

### MSRV

Do not add an MSRV job until workspace packages declare `rust-version`. Test the
declared minimum separately from stable.

### Release smoke tests

A post-release job may download each native artifact, verify checksums, extract
it, and run `--help`. It must not perform live AoC, editor, browser, or TUI
operations.

### Alternative test runners

Continue supporting `cargo test`. `cargo-nextest` may be added when suite size
justifies it, but it must not become the only supported local runner.

## Deferred tooling

Consider these only after target and artifact policies stabilize:

- artifact attestations;
- signing and notarization;
- `cargo-dist`;
- coverage gates;
- fuzzing in required CI;
- additional architectures;
- self-hosted runners.

## Other CI providers

GitHub Actions is canonical while the repository is hosted on GitHub.

Do not maintain parallel Jenkins, CircleCI, GitLab CI, or other provider
definitions without a concrete requirement. Reconsider only if hosting changes,
runner availability is insufficient, or release targets require dedicated
hardware.

## Branch protection

After workflows are stable, require:

- the quality job;
- every supported matrix entry;
- dependency review where available;
- review before changes to release workflows.

Release workflows run only from protected tags or the default branch.
