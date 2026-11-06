# indus-egm

Install the **Elephant-Goldfish Model** (EGM) into a coding agent from a single Rust binary.

EGM is Google's open-source design-review method: an *elephant* agent drafts a specification with your full project context, then fresh *goldfish* reviewers stress-test that draft with no memory of the conversation, so unstated assumptions surface before code is written. Upstream ships it as a plugin of Markdown skills and reviewer prompts. This crate vendors that bundle, embeds it in the binary at compile time, and copies it into whichever directory your agent reads.

Two crates in one, as `induscode` does it:

```sh
cargo install indus-egm                 # the CLI
cargo binstall indus-egm                # prebuilt binary, no compile
cargo add indus-egm                     # the library: indus_egm::<module>
```

> This is an installer and a bundle. It contains **no model weights** — EGM runs on whatever model your host agent already uses.

## CLI

```sh
indus-egm install --target induscode          # default target
indus-egm install --target claude-code
indus-egm install --dest ~/my-agent/skills/egm --force
indus-egm install --dry-run                   # show the file list, write nothing
indus-egm verify --target claude-code         # exit 2 if files are missing or edited
indus-egm list --sizes
indus-egm paths                               # default directory per target
```

Install rules: files identical to the embedded copy are left alone, files that differ locally are **skipped** unless you pass `--force`, and nothing outside `--dest` is ever written.

### Default destinations

| Target        | Directory                        |
| ------------- | -------------------------------- |
| `induscode`   | `~/.induscode/plugins/egm`       |
| `claude-code` | `~/.claude/plugins/egm`          |
| `codex`       | `~/.codex/plugins/egm`           |
| `antigravity` | `~/.antigravity/plugins/egm`     |

These defaults are conveniences, not contracts: agent harnesses move their plugin directories between releases, and this crate cannot verify yours. If your agent reads skills from somewhere else, pass `--dest` with the right path — that always works, because EGM itself is plain Markdown with no install step of its own.

## Library

```rust
use indus_egm::{assets, install, verify, InstallOptions, Target};

// Where would this target install to?
let home = std::path::PathBuf::from(std::env::var("HOME")?);
let dest = Target::IndusCode.default_dir(&home);

// Write the bundle, keeping any local edits.
let report = install(&InstallOptions::new(dest.clone()))?;
println!("{}", report.summary());

// Check the installed tree against the embedded copy.
if !verify(&dest)?.is_clean() {
    eprintln!("EGM is out of date at {}", dest.display());
}

// Read a file straight out of the binary.
let skill = assets::text("skills/egm-review/SKILL.md").unwrap();
# Ok::<(), std::io::Error>(())
```

Public surface: `assets` (`all`, `count`, `get`, `text`, `skill_names`, `agent_names`), `install` (`install`, `verify`, `safe_join`, `InstallOptions`, `InstallReport`, `VerifyReport`, `VerifyStatus`), and `target::Target`. The crate is `#![forbid(unsafe_code)]` and has one runtime dependency, `clap`.

## What gets installed

```
vendor/egm/
├── plugin.json          # upstream manifest (name: egm)
├── skills/              # egm-bootstrap, egm-brainstorm, egm-draft,
│                        # egm-drift-report, egm-resolve, egm-review, egm-review-loop
├── agents/              # 10 prompts: design_critic, structural, style, synthesizer, ...
└── resources/           # document standards + execution instructions
```

57 files, ~273 KB, embedded via a `build.rs`-generated table of `include_bytes!` calls, so the installed binary needs no network and no EGM checkout.

## Using it after install

EGM is driven from inside your agent, not from this CLI. In Claude Code and Antigravity the commands take an `egm:` prefix (`/egm-brainstorm`, `/egm-draft`, `/egm-review design_draft.md`); in Codex you pick the matching EGM skill. Reviews need an agent that can spawn isolated subagents — without that, EGM stops the review rather than faking one.

## Uninstall

Remove the directory it wrote, e.g. `rm -rf ~/.induscode/plugins/egm`, then `cargo uninstall indus-egm` if you also want the binary gone.

## Development

```sh
cargo build
cargo test            # bundle, install, idempotency, force, dry-run, verify
cargo clippy --all-targets -- -D warnings
```

To refresh the vendored bundle from upstream:

```sh
git clone --depth 1 https://github.com/google/elephant-goldfish-model /tmp/egm
rm -rf vendor/egm/skills vendor/egm/agents vendor/egm/resources
cp -r /tmp/egm/{skills,agents,resources,plugin.json,LICENSE} vendor/egm/
cargo build           # build.rs re-embeds whatever is on disk
```

## License and attribution

Apache-2.0, matching upstream. The vendored bundle in `vendor/egm` is from [google/elephant-goldfish-model](https://github.com/google/elephant-goldfish-model) at commit `6a8001ea31ccf72430ba0603a467ec6df9f69dd7`, © Google LLC, Apache-2.0. EGM is not an officially supported Google product. See `THIRD_PARTY_NOTICES.md`.
