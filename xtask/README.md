# xtask — MusterHub Developer Toolkit

`cargo xtask` is the workspace-native CLI for routine developer operations: scaffolding crates,
managing local infra, running tests, profiling, and generating licenses. Keep automation here
instead of ad-hoc scripts.

## Quick start

Run from the workspace root:

```sh
cargo xtask -h
cargo xtask setup
```

Prereqs:

- Rust toolchain (see `rust-toolchain.toml`)
- Docker (for `dev up/down/logs`)

## Commands

### Setup

```sh
cargo xtask setup
```

Installs developer tools, adds required rustup targets, and generates `private/keyset` if missing.

### Features

```sh
cargo xtask features add <name>
cargo xtask features list
```

Scaffold or list feature crates under `crates/features/` (names auto-prefixed with `mhub-`).

### Libraries (infra)

```sh
cargo xtask libs add <name>
cargo xtask libs list
```

Scaffold or list infra crates under `infra/` (names auto-prefixed with `mhub-`).

### Applications

```sh
cargo xtask apps add <name>
cargo xtask apps list
```

Scaffold or list apps under `apps/` (names auto-prefixed with `mhub-`).

### Development infra (Docker)

```sh
cargo xtask dev up             # start infra via ops/docker/mhub-dev/docker-compose.yml
cargo xtask dev down [-v]      # stop (optionally remove volumes)
cargo xtask dev logs [service] # follow logs
```

### Tests

```sh
cargo xtask test                    # runs all tests in the workspace
cargo xtask test all                # same as above
cargo xtask test server             # runs tests for `mhub-server`
cargo xtask test mhub-server        # same as above
cargo xtask doctest                 # runs doc tests in the workspace
cargo xtask doctest server          # runs doc tests for `mhub-server`
```

Runs the workspace test suite (prefers `cargo nextest` when available). Project names are
normalized with the `mhub-` prefix. Use `doctest` to run `cargo test --doc`.

### Run

```sh
cargo xtask run <crate-name>
```

Runs a project with `cargo run` (project names are normalized with the `mhub-` prefix).

### Bench

```sh
cargo xtask bench <crate-name>
```

Runs benches for a project with `cargo bench --all-features` (project names are normalized with the
`mhub-` prefix).

### Profiling

```sh
cargo xtask profiling --project <crate-name>  # runs a project with profiling
```

Runs a project with profiling (enables `tokio_unstable` flags and the `profiling` feature).

### License generation

```sh
cargo xtask lic --customer <name> --alias <short> --machines <ids|ANY> --matches <n> --features <list|*> --days <n>
```

Creates a signed license (uses `mhub-licensing` with the `issuance` feature). Use `--alias` for a
short namespace-friendly identifier.

## Tips

- Use `cargo xtask <command> -h` for detailed options.
- Override the composition file for tests/custom setups via `DockerCompose::with_file_path`.
- Add new automation as a handler + CLI arg, not ad-hoc scripts.
