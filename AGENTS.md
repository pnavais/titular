# Agent instructions (titular)

## Project

Titular is a Rust CLI (and library) for fancy terminal titles, themes, and related formatting. Source of truth for the toolchain is `rust-toolchain.toml`; the package is **edition 2021**.

## Prefer `just`

Use **[just](https://github.com/casey/just)** for repeatable workflows. If a step is awkward to type, add a **recipe** to `justfile` instead of pasting long `cargo` lines into chat or docs.

| Recipe | Purpose |
|--------|---------|
| `just build` | `cargo build` (pass extra args after recipe name) |
| `just b` | alias for `build` |
| `just build-full` | build with `--features display` |
| `just full` | alias for `build-full` |
| `just build-min` | `--no-default-features --features minimal` |
| `just min` | alias for `build-min` |
| `just fmt` | `cargo fmt` (all packages) |
| `just lint` | clippy with project warnings (`clippy:all`, `pedantic`) |
| `just lint-full` | clippy for `full_application` feature set |
| `just test` | `cargo test` (extra args supported) |
| `just check` | **fmt --check + lint + test** (default feature set); use before considering work done |
| `just c` | alias for `check` |
| `just clean` / `just release` / `just reset` / `just full_clean` | as in `justfile` |

When suggesting commands to the user, default to **`just …`** when a recipe exists.

## When to extend `justfile`

Add or adjust a recipe when:

- The same `cargo` incantation would be copied more than once (features, targets, doc tests, etc.).
- You need a stable name for “what CI / humans should run” (e.g. `just check`).

Keep recipes short and composable; reuse existing variables or patterns in `justfile`.

## Cargo directly

Use raw `cargo` only when there is no recipe yet—then prefer adding a `just` recipe for the next time.

Examples that may still be one-offs: `cargo doc --open`, `cargo install --path .`, or exploratory `cargo tree`.

## References

- `src/lib.rs` — library surface and module layout  
- `src/bin/titular/` — CLI entry and argument handling  
- `Cargo.toml` — features, bins, dependencies  
- `justfile` — automation entry points  

In Cursor rules/chats, you can `@`-mention these paths for concrete context.
