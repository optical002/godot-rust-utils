# godot-rust-utils

A monorepo of reusable utilities for building games with [Godot 4](https://godotengine.org/) and Rust via the [godot-rust](https://github.com/godot-rust/gdext) (`gdext`) crate.

## Crates

| Crate | Description |
|-------|-------------|
| [`init-system`](crates/init-system) | Lifecycle and initialization management for game objects |

## Structure

```
crates/
  init-system/          # Standalone crate, published to crates.io
  init-system-macros/   # Proc-macro companion for init-system
```

Each crate is independent and versioned separately. They are published to [crates.io](https://crates.io) individually.

## CI

Every push and pull request runs `cargo check`, `cargo clippy`, and `cargo test` (where possible). Tests for crates that depend on Godot native libraries cannot run in headless CI and are excluded — see individual crate READMEs for details.

## License

MIT
