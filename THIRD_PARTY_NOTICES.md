# Third-party notices

This repository is an independent installer. It is not affiliated with, or
endorsed by, Google or the REA maintainers.

## Elephant-Goldfish Model (EGM)

The contents of `vendor/egm` are copied unmodified from:

- Repository: https://github.com/google/elephant-goldfish-model
- Commit: `6a8001ea31ccf72430ba0603a467ec6df9f69dd7`
- Copyright: Google LLC
- License: Apache License 2.0 (full text in `vendor/egm/LICENSE`)

Included paths: `skills/`, `agents/`, `resources/`, `plugin.json`, `LICENSE`.
EGM is not an officially supported Google product.

## REA: Reverse Engineer Anything

The contents of `vendor/rea/skills` are copied unmodified from:

- Repository: https://github.com/morluto/rea
- Commit: `72d5e72e6d7b7997c4ecb11972fc405be8ec67d2`
- Version: 6.8.0 (`rea-agents` on npm)
- Copyright: 2026 morluto
- License: MIT (full text in `vendor/rea/LICENSE`)

Included paths: `skills/` (five skills), `LICENSE`, and a `SOURCE.json`
provenance file written by this project.

**Not redistributed here:** REA's TypeScript source, CLI, MCP server, website,
tests and its `third_party` git submodules (jadx-headless-mcp, binwalk, unblob,
ghidra-nativeaot, wakaru). Those are fetched from npm at run time as
`rea-agents@6.8.0`. The MIT licence permits redistribution; this project simply
does not need to, and keeping a 94 MB runtime out of the binary is the point.

## Dependencies

- `clap` — MIT OR Apache-2.0
- `tempfile` (dev only) — MIT OR Apache-2.0
