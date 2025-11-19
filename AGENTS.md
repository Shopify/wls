AGENTS guide for eza (Rust)
Build: `cargo build`; release: `cargo build --release`
Or `just build` / `just build-release`; Nix: `nix build .#default`
Test all: `just test` or `cargo test --workspace -- --quiet`
Single test: `cargo test <pattern>` (filters by module/name)
Trycmd single case: `cargo test --test cli_tests <file-stem>`
Nix trycmd: `just itest` (sandbox) or `nix build .#trycmd`
Lint: `just clippy` or `cargo clippy` (fix warnings)
Format: `cargo fmt --all` or `nix fmt` (treefmt)
Pre-commit (Nix): `nix develop -c pre-commit run -a`
Rust: edition 2021; MSRV 1.82.0; rustfmt defaults
Imports: `std` -> external -> `crate`/`super`; keep minimal
Naming: Types/Traits UpperCamelCase; funcs/vars/modules snake_case; consts SCREAMING_SNAKE_CASE
Errors: use `Result<T, E>`; prefer enums with `Display` (see `src/options/error.rs`)
Avoid panics; no `unwrap/expect` outside tests; use `?` and helpful messages
Logging: use `log` macros; no println!; init via `logger::configure(...)`
Types: prefer `&str` over `String` when possible; use `OsStr/OsString` for OS paths
Performance: avoid unnecessary clones; iterate; use `rayon` consciously
Features: default enables `git`; test extras with `--features nix-local` etc.
Before PR: run clippy, fmt, tests; keep diffs focused