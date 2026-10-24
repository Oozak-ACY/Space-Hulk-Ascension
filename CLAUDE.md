# Space Hulk Ascension

2D twin-stick roguelike in Rust + Bevy (WebAssembly). Design source of truth: `WH40K_Roguelike_Tech_Spec_v3.md`.

## Build & run

Bevy is pinned to 0.20.0 (current release when M1 was scaffolded, October 2026).

- Native: `cargo run`
- Browser: `trunk serve --release` (debug wasm is too large to serve comfortably)
- Tests: `cargo test` — they drive `GameplayPlugin` through the headless app in `src/headless.rs`; the client plugin (`src/client/`) is not tested.

## Agent skills

### Issue tracker

Issues and specs live in GitHub Issues on Oozak-ACY/Space-Hulk-Ascension, via the `gh` CLI. See `docs/agents/issue-tracker.md`.

### Triage labels

Default five-role vocabulary: needs-triage, needs-info, ready-for-agent, ready-for-human, wontfix. See `docs/agents/triage-labels.md`.

### Domain docs

Single-context: `CONTEXT.md` + `docs/adr/` at the repo root. See `docs/agents/domain.md`.
