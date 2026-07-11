# Developer Guide

## xtask automation

TerminalOS ships a `cargo xtask` helper for common development workflows:

```bash
cargo xtask ci          # fmt check, clippy, tests, snapshots
cargo xtask fmt         # format all crates
cargo xtask clippy      # lint with warnings denied
cargo xtask test        # run workspace tests
cargo xtask snapshot    # verify snapshot tests
cargo xtask snapshot --update  # refresh snapshot files
cargo xtask bench       # run criterion benchmarks
cargo xtask docs        # build mdBook site to docs/book/
cargo xtask hooks       # install commit-msg git hook
```

## Quality gates

- Zero clippy warnings: `cargo clippy --workspace --all-targets -- -D warnings`
- Formatting: `cargo fmt --all`
- Tests: `cargo test --workspace`
- Snapshots: `cargo xtask snapshot`

## Snapshot tests

Snapshot tests live in:

- `crates/config/tests/snapshots.rs` — default config TOML, keybindings, themes
- `crates/ui/tests/snapshots.rs` — layout geometry and status bar rendering

Update snapshots after intentional UI or config changes:

```bash
cargo xtask snapshot --update
```

## Benchmarks

Criterion benchmarks:

- `crates/config/benches/keybinding.rs` — keybinding parser throughput
- `crates/search/benches/search.rs` — Tantivy keyword search latency

Run with:

```bash
cargo xtask bench
```

## Documentation site

Build the mdBook site locally:

```bash
cargo install mdbook
cargo xtask docs
open docs/book/index.html
```

## Contributing

See [CONTRIBUTING.md](https://github.com/panditdhamdhere/WarpShell/blob/main/CONTRIBUTING.md) for code standards, commit policy, and PR workflow.
