# AGENTS.md

## Build/Test Commands

- Build: `cargo build` (debug) or `cargo build --release`
- Run tests: `cargo test --verbose`
- Run single test: `cargo test test_name --verbose` (e.g., `cargo test test_parse_run_command`)
- Lint: `cargo clippy -- -D warnings`
- Format check: `cargo fmt --all -- --check`
- Format fix: `cargo fmt --all`

## Code Style Guidelines

- **Rust Edition**: 2021
- **Formatting**: Use `rustfmt` defaults (run `cargo fmt` before committing)
- **Linting**: All clippy warnings treated as errors (`-D warnings`)
- **Imports**: Group std imports first, then external crates (clap, colored, regex), use specific imports
- **Naming**: snake_case for functions/variables, PascalCase for types, SCREAMING_SNAKE_CASE for constants
- **Error Handling**: Use `expect()` with descriptive messages for unrecoverable errors; print colored error messages with `eprintln!` using the `colored` crate (red for errors, yellow for hints)
- **Tests**: Place in `src/tests.rs` as a separate module; use `#[cfg(test)]` conditional compilation
- **CLI**: Use `clap` with derive features for argument parsing
