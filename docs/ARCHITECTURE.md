# Architecture

## Shape

One crate, two faces:

```text
indus-egm (bin)  ──> indus_egm::install   ──> <dest>/<bundle>/...
indus-egm (lib)  ──> same functions, callable from Rust
```

`src/main.rs` is deliberately thin: it parses flags, resolves a destination,
calls the library, and formats output. Everything that decides *what gets
written* lives in the library, which is why the tests can cover install,
idempotency, force semantics and migration without spawning a binary.

## Modules

| Module | Responsibility |
| ------ | -------------- |
| `bundle` | `BundleId` (Egm, Rea), their metadata, and `Runtime` requirements |
| `assets` | The compile-time embedded file table, filtered by bundle |
| `generated` | Deterministic files produced at install time (`rea/mcp.json`) |
| `install` | `install`, `verify`, `migrate`, `safe_join`, report types |
| `target` | Harness names and their default directories |

## Embedding

`build.rs` walks `vendor/egm` and `vendor/rea`, sorts the results, and writes
`$OUT_DIR/assets_generated.rs` — a static table of `Asset { bundle, path, bytes }`
where every `bytes` is an `include_bytes!` of the canonical file path.

Consequences worth knowing:

- The binary carries the bundle; `install` needs no network and no checkout.
- Adding a vendored file needs no code change — only `cargo build`.
- `cargo:rerun-if-changed` is emitted per vendor directory, so edits re-embed.
- Paths are stored bundle-relative with `/` separators, which keeps the table
  platform-independent and lets `safe_join` reject absolute paths and `..`.

## Why generated files are part of the same contract

`rea/mcp.json` is not vendored, but it must still be verifiable. So
`generated::for_bundle` returns content that is a pure function of the crate's
constants — no clock, no environment, no randomness. `verify` byte-compares it
exactly like an embedded asset, and `install` treats it identically for
skip/force/dry-run purposes. One code path, one set of rules, no special cases.

## Install semantics

For every expected file:

1. Read what is on disk (absent is fine).
2. Identical → `unchanged`, nothing written.
3. Different and no `--force` → `skipped`, nothing written, reported.
4. Different with `--force`, or absent → written (or listed, on `--dry-run`).

Nothing outside the destination is touched, and `safe_join` rejects any asset
path that is absolute or contains a parent component — a guard that matters
because the asset table is generated from a directory walk.

## Migration

`migrate` handles the 0.1.x layout where EGM files sat at the destination root.
It moves a file only when its bytes match the embedded copy; anything edited
stays put and is listed in `MigrateReport::left`. It refuses to run when
`<dest>/egm` already exists, so it cannot half-migrate a tree twice.

## REA boundary

The rule that keeps this crate honest: **embed what the agent reads, fetch what
the machine runs.** REA's skills are Markdown the agent loads, so they are
vendored. Its MCP server is a Node program with native analysers behind it, so
it is started via `npx -y rea-agents@6.8.0 mcp` and never copied in. `doctor`
checks the Node range REA publishes, because a mismatched Node version is the
most likely reason the server will not start.
