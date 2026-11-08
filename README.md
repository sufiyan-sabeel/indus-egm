# indus-egm

Install agent bundles into a coding agent from one Rust binary.

Two bundles ship together: **EGM** for designing software before you build it, and **REA** for understanding software that already exists. One binary, one `cargo install`, one destination directory — but each bundle keeps its own root, licence and runtime story.

| Bundle | What it does | Upstream | Licence | Runtime |
| ------ | ------------ | -------- | ------- | ------- |
| `egm` | Elephant-Goldfish Model — draft a spec with full context, then stress-test it with fresh-context reviewers | [google/elephant-goldfish-model](https://github.com/google/elephant-goldfish-model) | Apache-2.0 | none |
| `rea` | REA: Reverse Engineer Anything — skills plus an MCP server for binaries, apps and runtime behaviour | [morluto/rea](https://github.com/morluto/rea) | MIT | Node ^22.23.4 |

```sh
cargo install indus-egm                 # the CLI
cargo binstall indus-egm                # prebuilt binary, no compile
cargo add indus-egm                     # the library: indus_egm::<module>
```

> Neither bundle contains model weights. They run on whatever model your host agent already uses.

## Install

```sh
indus-egm install                              # both bundles, ~/.induscode/plugins/egm
indus-egm install --bundle egm                 # one bundle
indus-egm install --bundle rea --target claude-code
indus-egm install --dest ~/agent/plugins --force
indus-egm install --dry-run                    # list files, write nothing
```

Files land under a per-bundle root, so the two never collide and each can be removed on its own:

```text
<dest>/
├── egm/
│   ├── plugin.json          # upstream manifest
│   ├── skills/              # egm-bootstrap, egm-brainstorm, egm-draft, egm-drift-report,
│   │                        # egm-resolve, egm-review, egm-review-loop
│   ├── agents/              # 10 reviewer prompts
│   └── resources/           # document standards + execution instructions
└── rea/
    ├── skills/              # reverse-engineer-anything, rea-tool-design,
    │                        # rea-ci-tests-audit, rea-analysis-optimization, rea-changelog-update
    ├── mcp.json             # generated: MCP server registration
    ├── SOURCE.json          # provenance of the vendored files
    └── LICENSE              # MIT
```

Install rules: identical files are left alone, locally edited files are **skipped** unless you pass `--force`, and nothing outside `--dest` is ever written.

## Why REA's runtime is not vendored

REA upstream is a 94 MB TypeScript monorepo — 6,032 `.ts` files, five git submodules for jadx, binwalk, unblob, ghidra-nativeaot and wakaru, and a Node 22+ runtime. Copying that into a Rust installer would make the crate unusable and would still not run without Node.

So this crate embeds REA's **agent-facing** surface — its five skills, provenance and licence, 233 KB in total — and generates the MCP registration that starts the real server from npm, pinned to the vendored version:

```json
{
  "mcpServers": {
    "rea": {
      "command": "npx",
      "args": ["-y", "rea-agents@6.8.0", "mcp"],
      "startup_timeout_sec": 120
    }
  }
}
```

Merge that block into your agent's MCP configuration, or let REA's own installer do it for you with `npx rea-agents setup`, which also backs up existing config. The first `npx` run downloads the server, so it needs network once.

## Commands

```sh
indus-egm install [--bundle LIST] [--target T | --dest DIR] [--force] [--dry-run]
indus-egm verify  [--bundle LIST] [--target T | --dest DIR]   # exit 2 on drift
indus-egm list    [--bundle LIST] [--sizes]
indus-egm bundles                                              # versions, licences, runtimes
indus-egm doctor  [--strict]                                   # is Node new enough?
indus-egm migrate [--dest DIR]                                 # v0.1.x flat layout -> bundle layout
indus-egm paths
```

`--bundle` accepts `egm`, `rea`, `all`, or a comma-separated list; the default is `all`.

### Default destinations

| Target        | Directory                    |
| ------------- | ---------------------------- |
| `induscode`   | `~/.induscode/plugins/egm`   |
| `claude-code` | `~/.claude/plugins/egm`      |
| `codex`       | `~/.codex/plugins/egm`       |
| `antigravity` | `~/.antigravity/plugins/egm` |

These defaults are conveniences, not contracts: harnesses move their plugin directories between releases. Pass `--dest` with the path your agent actually reads.

### Upgrading from 0.1.x

0.1.x wrote EGM files straight into the destination root. 0.2.0 writes them under `<dest>/egm/`. `indus-egm migrate --dest <your old dir>` moves every file that still matches the embedded copy byte for byte, and leaves anything you edited exactly where it is, reporting each one.

## Library

```rust
use indus_egm::{assets, install, verify, BundleId, InstallOptions};

let options = InstallOptions::new("/home/me/.induscode/plugins")
    .bundles(vec![BundleId::Egm, BundleId::Rea]);
let report = install(&options)?;
println!("{}", report.summary());

// Narrow checks, or read a file straight out of the binary.
assert!(verify("/home/me/.induscode/plugins", &[BundleId::Rea])?.is_clean());
let skill = assets::text(BundleId::Rea, "skills/reverse-engineer-anything/SKILL.md").unwrap();
# Ok::<(), std::io::Error>(())
```

Public surface: `assets` (`all`, `by_bundle`, `count`, `count_in`, `get`, `text`, `skill_names`, `agent_names`), `bundle` (`BundleId`, `Runtime`), `generated` (`for_bundle`, `rea_mcp_json`, `GeneratedFile`), `install` (`install`, `verify`, `migrate`, `safe_join`, `InstallOptions`, `InstallReport`, `VerifyReport`, `VerifyStatus`, `MigrateReport`), and `target::Target`. The crate is `#![forbid(unsafe_code)]` with one runtime dependency, `clap`.

## Using the bundles

EGM is driven from inside your agent — `/egm-brainstorm`, `/egm-draft`, `/egm-review design_draft.md` in Claude Code and Antigravity, or the matching skill in Codex. Reviews need an agent that can spawn isolated subagents; without that, EGM stops rather than faking a review.

REA answers questions through its MCP server once registered, e.g. "understand how search works in this app, show me the evidence". Its tooling leans on external analysers (Ghidra, JADX, binwalk) that you install separately; on Linux it also wants a procps-compatible `ps` on `PATH`.

## Development

```sh
cargo build
cargo test
cargo clippy --all-targets -- -D warnings
cargo run -- bundles
cargo run -- doctor
```

Refreshing the vendored bundles:

```sh
git clone --depth 1 https://github.com/google/elephant-goldfish-model /tmp/egm
rm -rf vendor/egm/skills vendor/egm/agents vendor/egm/resources
cp -r /tmp/egm/{skills,agents,resources,plugin.json,LICENSE} vendor/egm/

git clone --depth 1 https://github.com/morluto/rea /tmp/rea
rm -rf vendor/rea/skills
cp -r /tmp/rea/.agents/skills vendor/rea/skills
cp /tmp/rea/LICENSE vendor/rea/LICENSE
# then update vendor/rea/SOURCE.json and the REA_* constants in src/lib.rs

cargo build     # build.rs re-embeds whatever is on disk
```

## Licence and attribution

This crate is Apache-2.0. `vendor/egm` is Apache-2.0, © Google LLC, taken from commit `6a8001e`; EGM is not an officially supported Google product. `vendor/rea` is MIT, © 2026 morluto, taken from commit `72d5e72`. Both are unmodified copies of their upstream agent assets. This repository is an independent installer, not affiliated with or endorsed by either upstream. See `THIRD_PARTY_NOTICES.md`.
