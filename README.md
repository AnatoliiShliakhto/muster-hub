# MusterHub 🛡️

![Rust](https://img.shields.io/badge/built_with-Rust-dca282.svg)
![No Unsafe](https://img.shields.io/badge/unsafe-forbidden-success.svg)
![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)
![Version](https://img.shields.io/badge/version-0.0.0-green.svg)
![GitHub Workflow Status](https://img.shields.io/github/actions/workflow/status/AnatoliiShliakhto/muster-hub/ci.yml?branch=dev)

**MusterHub** is a high-performance orchestration platform for secure data collection and task
management, running as a classic online server with device synchronization or as a portable,
laptop-hosted hub that can operate offline. Built entirely in **Rust**, it lets you deploy a
centralized server or spin up a localized “Hub” on a laptop. Clients connect over the network—online
or via a
local Wi-Fi mesh—so slice-defined entities *`(Features)`* stay synchronized with or without external
cloud access

## Status

> **⚠️ Project Status: Early Development (WIP)**  
> This project is currently in its initial development phase. Core architectures are being
> established, and features are subject to frequent breaking changes. **Not ready for production
> use.**

## Audience & Prereqs

- Rust toolchain from `rust-toolchain.toml`.
- Docker (for `cargo xtask dev`).

## Workspace Layout

```text
.
├── apps/               # Entrypoints & Binaries
│   ├── mhub-server     # Axum-based API server
│   ├── mhub-desktop    # Dioxus-based UI
│   └── mhub-shell      # Standalone App Shell
├── crates/             # Business Logic
│   ├── features/       # Domain slices
│   └── shared/         # Common types (Kernel, Domain)
│   └── mhub            # Facade
├── infra/              # Infrastructure Adapters
├── xtask/              # Developer Automation CLI
└── assets/             # Branding, Icons, and Static Configs
```

* Per-crate docs: see `infra/*/README.md`, `crates/*/README.md`, `apps/*/README.md`.

## Quick Start

```sh
# Clone and setup in one go
git clone https://github.com/AnatoliiShliakhto/muster-hub.git
cd muster-hub
cargo xtask setup

# Run the Shell
cargo xtask run mhub-shell
```

## Configuration

- Example (`server.toml`):
- Env overrides: prefix `MHUB__`, nested with `__` (e.g., `MHUB__DATABASE__URL`).
- SSL: set `server.ssl.cert` / `server.ssl.key`; warns on permissive key perms (Unix).

```toml
[server]
address = "::"
port = 4583

[database]
url = "mem://"
namespace = "mhub"
database = "core"
```

## Architecture

### Vertical Slice Architecture (VSA)

![Architecture](https://kroki.io/plantuml/svg/eNqNVttu2zgQfedXcB1g031wmrZO7BRGUFmXNtgkNpxuX4KgoCXaIiyRAkmlMbr59x2Skqw41sZ-GvLwzH1G_qI0kbrMM6TWjBdEkhxnjFO9KSgWUqeiBaiUJOIX4yusZUlbgBQlT2IhOZX4rHW_FFxzklPc-0FlQjjpIfRHQpdgAHuz2U9_ej2d46PgQ-hFUYNcT79e-TUWjSIv8hssCr3v_8zDGg0_hqPotEH96RaaeOF5FDTQ1W009xqlH6Kz6KLBJtN5EM5r8HwwHIwmW0eD6Y13dQvoLT46_Tga-MNXSgG5GHifJqM9fgI2GJ6dnV8g1MpMLPJCcMo1_o0w_CYkXq9sGn2RCYnH45BrucEzwbi-vMRHwyD4BHmwb4VMqHTv2q6jZ4RSSgDDN6XSVH4rF9iTcco0jXUpIT9QD_qk0VIIgPFXCgUjmiZQKDgxbQj_Yo8TLTLG7tKMkXWqxfvcquun5QKhAjwlKyjpTFIFARDNBMfXZENlDxOFX1wfRfZXBbkN-v4upVmG33lF8deDYSl73ol6lxRQtdaiaNESd_OTFMVb5DsqHyHmlkl7cQg1h8D7SW08YOKpVC_sg4JImtzyBGrVdPZePar246nM246ADtMEuyqeWxmfQi2p0tLl9k98LVYstkm3Une2jWH8LiIxNIezaW_G461GIW2XDfxT6DLLb8xGlJjuUb16cCoD-6LLWEy5ghVhrbgTBUMm-3AAGy8muFMPS0BmemPVkFKnb-h43vE5EDlhJke-kPQQx9cUtldmzTnRNsUKZhnsbfdKJz-xBl1TONvjcfB9-j40YTCqXmt5flHbK76UBCpRukndzpMF_r-0_YRosiCKOvOmsDMqFYOh5TZfreW3l_9IykxbspVsruNSQv4PICvoHQjBtbKTzTywjKoNeJAfoII-gtxflMoqsSdl5qFUB5AzsVpR6frNisD0pVCqH5daQyuaxnbp28uXJZQod_5XMmjw1IbHeO7Ob2iAjcseq-RbEfg3JAYfDDE8DYdAhGIfY1g-CrldBx3zi_cv6xWyc-t2AkLtBfeKsV1gHbTm3o77Z9w7jjMGfh_jpZvpXqXk9UN333oI3rsFguybk_vWJ-4BmNWsd6BmhB3Uv99-UA3ips2o_9tJ1fjtvnNTZd2o9hGq18tJx1O7N07226uw5gv-cHJpJgdCt6G6r0WdnZ13VZd0oK4JO8Bk0QFUk9OB2rHswNy4oC_w5YD_cf8Brckm2A==)

* See the [ARCHITECTURE.md](ARCHITECTURE.md) file for details.

## Feature Flags

- Server: `--features server` (Axum & API slices).
- Client: `--features client` (client-side pieces).
- Profiling: `--features profiling` (tokio-console instrumentation and dhat heap tracking).
- Licensing: `--features issuance` (license generation routine).

## Development Workflow

```sh
cargo format                        # format everything
cargo lint                          # clippy lint everything
```

## Xtask (Developer CLI)

```sh
cargo xtask --help
cargo xtask setup                   # install/update dev environment & tooling
cargo xtask dev up|down|logs        # docker-compose infra
cargo xtask features|libs|apps ...  # scaffold/list crates
cargo xtask test [<crate>|all]      # run workspace or crate tests
cargo xtask doctest [<crate>|all]   # run workspace or crate doc tests
cargo xtask run <crate>             # run a project
cargo xtask bench <crate>           # run benches for a project
cargo xtask profiling --project X   # run with profiling flags (dhat + tokio-console)
cargo xtask lic ...                 # generate signed license (issuance)
```

* See the [xtask/README.md](xtask/README.md) file for details.

## Testing & CI

- Local: fmt, check, clippy, tests, benches, profiling (dhat + tokio-console).
- Docker:.
- CI (GitHub Actions): fmt, check, clippy, tests, coverage (lcov/Codecov), docs deploy,
  cargo-deny/audit (scheduled), dependency review. Lint level `-D warnings`.

## Notable Crates

- Facade: `crates/mhub`.
- Shared: `crates/shared/kernel`, `crates/shared/domain`.
- Features: `crates/features/*`.
- Infra: `infra/*`.

## Troubleshooting

- SSL paths: ensure cert/key exist; Unix warns on permissive key permissions.

## License

This project is dual-licensed under the **MIT License** and the **Apache License (Version 2.0)**.
You may choose to use this software under the terms of either license.

* See [LICENSE-MIT](LICENSE-MIT) for details.
* See [LICENSE-APACHE](LICENSE-APACHE) for details.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the
work by you, as defined in the Apache-2.0 license, shall be dual-licensed as above, without any
additional terms or conditions.

* See [CONTRIBUTING.md](CONTRIBUTING.md) for details.
