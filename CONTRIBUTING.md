# Contributing to fehrust

Thanks for considering a contribution. fehrust is a Windows-only, native Rust
image viewer, so changes should preserve the Windows user experience and avoid
adding a cross-platform GUI dependency unless there is a compelling reason.

## Before you start

- Search existing issues and pull requests before opening a new one.
- For substantial behavior or architecture changes, open an issue first so the
  approach can be discussed.
- Keep changes focused and update the README or changelog when user-visible
  behavior changes.

## Development

Use a current stable Rust toolchain on Windows. Before submitting a pull
request, run:

```powershell
cargo fmt -- --check
cargo clippy -- -D warnings
cargo test
cargo build --release
```

The test suite includes pure-logic unit tests and Windows CLI/integration
tests. New behavior should include focused tests where practical, especially
for file-list, parsing, formatting, and filesystem operations.

To inspect line coverage locally, install `cargo-llvm-cov` and run:

```powershell
cargo llvm-cov --all-features --workspace --summary-only
```

CI generates the same summary and stores an LCOV report as a workflow
artifact. Coverage is a signal for missing behavior, not a substitute for
reviewing error paths and Windows-specific integration behavior.

## Pull requests

Please include:

- A concise description of the problem and solution.
- The user-visible impact, if any.
- Tests run and any Windows-specific limitations.
- Documentation or changelog updates for user-facing changes.

Pull requests should pass the required CI checks and remain compatible with
the supported Windows target and stable Rust toolchain.
