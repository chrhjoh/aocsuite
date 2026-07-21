# Continuous Integration And Releases

## Goals

Use GitHub Actions to verify every pull request, exercise supported operating systems, and publish reproducible CLI binaries. Add TUI binaries to the same release process after `aocsuite-tui` exists and reaches command parity.

CI must never use a real AoC session, contact Advent of Code, submit answers, launch user applications, or depend on developer-local runtime state. Tests use explicit temporary roots and fake HTTP, process, clock, and environment seams.

## Rollout

Introduce CI in stages so required checks reflect a passing repository baseline.

### Stage 1: Baseline CI

Add `.github/workflows/ci.yml` on pull requests and pushes to the default branch with:

- `cargo check --workspace --locked`
- `cargo test --workspace --locked`
- `cargo run -p aocsuite-cli --locked -- --help`

Run this initial job on Ubuntu with an explicit temporary `XDG_DATA_HOME` because the target application bootstraps storage on every invocation. Do not make formatting or strict Clippy required until the existing unrelated failures recorded in `IMPLEMENTATION_NOTES.md` are fixed.

Use workflow concurrency to cancel superseded runs for the same branch. Grant read-only repository permissions and no secrets.

### Stage 2: Cross-Platform Matrix

After process, environment, HTTP, and filesystem tests use deterministic fakes, run stable Rust on:

- Ubuntu x86-64.
- Windows x86-64.
- macOS ARM64.

Install a supported Python version through `actions/setup-python` for tests that inspect Python project behavior, but normal deterministic coverage must not create real virtual environments or invoke pip. Keep optional real Cargo/Python smoke tests in a separate non-required workflow.

Each matrix entry runs:

```text
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo run -p aocsuite-cli --locked -- --help
```

After `aocsuite-tui` exists, add a noninteractive TUI compile/test target. Do not launch a real terminal UI in CI; use `ratatui::TestBackend` and fake terminal operations.

### Stage 3: Quality Gates

Once their baselines pass, add required jobs for:

```text
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps
```

Keep these separate from the OS matrix so failures are easy to classify and redundant work is limited.

## Suggested CI Workflow

`.github/workflows/ci.yml` should contain these jobs:

### `quality`

- Ubuntu stable Rust.
- Formatting, Clippy, and documentation after their baselines are clean.
- No runtime secrets.

### `test`

- Matrix across Ubuntu, Windows, and macOS.
- Workspace check and tests with `--locked`.
- Python installed explicitly where needed.
- CLI help smoke test.
- TUI unit/render tests after the crate exists.

### `feature-boundaries`

- Verify the workspace compiles without frontend-only features leaking into shared crates where practical.
- Verify no normal test requires `AOC_SESSION`.
- Verify generated test roots remain inside temporary directories.

The boundary checks may initially be ordinary Rust tests rather than shell scripts. Prefer behavior assertions in the owning crate.

Recommended actions:

- `actions/checkout`
- `dtolnay/rust-toolchain`
- `Swatinem/rust-cache`
- `actions/setup-python`

Pin third-party actions to immutable commit SHAs and configure Dependabot to update GitHub Actions references.

## Release Versioning

Use one product version across CLI, future TUI, and internal workspace crates while they are released together. A release tag is `vMAJOR.MINOR.PATCH` and must match the package versions in the workspace.

Do not automatically publish crates to crates.io as part of the initial binary release. Path-connected internal crates require an explicit publication policy and ordering if crates.io distribution is added later.

Release preparation should:

1. Update all workspace package versions consistently.
2. Update `Cargo.lock`.
3. Record user-visible changes in release notes or a changelog.
4. Merge through normal required CI.
5. Create an annotated `vMAJOR.MINOR.PATCH` tag.

## Suggested Release Workflow

Add `.github/workflows/release.yml` triggered by version tags and optionally `workflow_dispatch` for a dry run.

The workflow must:

1. Validate that the tag matches the workspace product version.
2. Run or require the complete CI suite.
3. Build with `cargo build --release --locked` for each release target.
4. Execute each native binary with `--help` before packaging.
5. Package the binaries with README and license files.
6. Generate SHA-256 checksum files for release artifacts.
7. Create a GitHub Release and upload all archives and checksums.

Initial CLI targets:

```text
x86_64-unknown-linux-gnu
x86_64-pc-windows-msvc
aarch64-apple-darwin
```

Add other targets only when they have a tested build strategy. Prefer native GitHub runners initially; introduce `cross`, Zig, or custom Docker images only for targets that cannot be built natively.

Suggested artifact names:

```text
aocsuite-cli-v0.4.0-x86_64-unknown-linux-gnu.tar.gz
aocsuite-cli-v0.4.0-x86_64-pc-windows-msvc.zip
aocsuite-cli-v0.4.0-aarch64-apple-darwin.tar.gz
```

After the TUI is release-ready, include both executables in each platform archive or publish parallel CLI/TUI archives. Prefer one product archive when both binaries always share a version; use separate archives only if installation or platform support differs.

The release workflow needs `contents: write` only for its release job. Build/test jobs remain read-only. GitHub artifact attestations are useful once binary releases stabilize; platform code signing and notarization can be added when distribution requirements justify their secrets and maintenance.

`cargo-dist` is worth considering after target support and artifact layout stabilize. Explicit workflows are easier to understand during the first release iterations; `cargo-dist` can later generate release jobs, installers, checksums, and GitHub Release integration.

## Additional Automation

### Dependency And License Policy

Add `.github/workflows/security.yml` on a weekly schedule and dependency-changing pull requests:

```text
cargo deny check advisories bans licenses sources
```

Use `cargo-deny` rather than overlapping `cargo-audit` and custom license scripts. Add `.github/dependabot.yml` for Cargo and GitHub Actions updates.

### Coverage

An optional Linux-only workflow can run `cargo llvm-cov --workspace --lcov`. Uploading to Codecov is useful for trends but should not block merges initially. Prefer behavior coverage targets over a global percentage gate.

### Parser Fuzzing

AoC HTML is external input. Scheduled or manually triggered `cargo-fuzz` targets for puzzle, calendar, and submission parsers may be valuable after the parser APIs become pure and fixture coverage is established. This is lower priority than deterministic parser tests.

### Supply-Chain And Static Analysis

GitHub dependency review is useful on pull requests. CodeQL may be enabled if Rust support and repository settings provide useful results, but it is secondary to Clippy, `cargo-deny`, and focused tests for this codebase.

### MSRV

Do not add an MSRV job until `rust-version` is declared for workspace packages. Once declared, test the minimum supported toolchain separately from stable.

### Release Smoke Tests

After publishing a GitHub Release, a small follow-up job may download each native artifact, verify its checksum, extract it, and run `--help`. Do not perform live AoC or editor/browser smoke tests.

### Test Runner Alternatives

Continue using `cargo test` initially. `cargo-nextest` can reduce runtime and improve reporting once the suite is large enough to justify installing another CI tool, but it should not be the only supported way to run tests locally.

## Other CI Providers

Do not maintain parallel Jenkins, CircleCI, GitLab CI, or other hosted-CI definitions while GitHub is the canonical repository. Multiple providers would duplicate secrets, caches, platform policy, and release permissions without improving the initial support matrix.

Reconsider another provider only if the repository moves, GitHub-hosted runner availability becomes insufficient, or release targets require dedicated hardware. Self-hosted runners should likewise be avoided until signing, notarization, or unsupported architecture builds require them.

## Branch Protection

After the workflows are stable, require:

- The quality job.
- Every supported test-matrix entry.
- Dependency review where available.
- Review before merging changes to release workflows.

Release workflows run only from protected tags or the default branch and must not execute untrusted pull-request code with write permissions.
