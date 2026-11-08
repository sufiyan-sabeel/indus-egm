# Changelog

## 0.2.0 — mixed bundles

Breaking: the on-disk layout changed. Files now live under a per-bundle root,
`<dest>/egm/...` and `<dest>/rea/...`, instead of being written at the
destination root. `indus-egm migrate --dest <dir>` moves a 0.1.x install across
without touching files you edited.

Added:

- **REA bundle** (`morluto/rea` 6.8.0, MIT): five agent skills, provenance and
  licence embedded; a `rea/mcp.json` registration generated at install time that
  starts `npx -y rea-agents@6.8.0 mcp`. The 94 MB TypeScript runtime and its five
  git submodules are deliberately not vendored.
- `--bundle egm|rea|all` on `install`, `verify` and `list`; default is both.
- `indus-egm bundles` — versions, commits, licences, file counts, runtimes.
- `indus-egm doctor [--strict]` — checks `HOME` and REA's Node requirement
  (`^22.23.4 || ^24.22.1 || >=26.12.1`), exiting 3 with `--strict` when unmet.
- `indus-egm migrate` — safe, non-destructive move of a 0.1.x flat install.
- `bundle`, `generated` modules and `BundleId`/`Runtime`/`GeneratedFile` types.

Changed:

- `InstallOptions` gained a `bundles` field; `verify` now takes a bundle slice.
- Generated files are deterministic functions of the crate constants, so
  `verify` byte-compares them exactly like vendored assets.

## 0.1.0 — EGM only

- Embedded the Elephant-Goldfish Model bundle (57 files, Apache-2.0).
- `install`, `verify`, `list`, `paths` with `--target`/`--dest`, `--force`,
  `--dry-run`.
- Targets: `induscode`, `claude-code`, `codex`, `antigravity`.
