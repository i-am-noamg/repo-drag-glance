# Integration tests

- [`cli_integration.rs`](cli_integration.rs) — temp git repo, real `git` and
  `repo-drag-glance` binary (`CARGO_BIN_EXE_repo_drag_glance`). Needs a normal
  environment where `git init` can create `.git/hooks` (some sandboxes block
  that).

Unit tests live next to the code under `src/` (for example `git::tests`,
`alerts::tests`, `report::tests`).
