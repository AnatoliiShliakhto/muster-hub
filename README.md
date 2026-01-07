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

![Architecture](https://kroki.io/plantuml/svg/eNqdVl1z2jgUfdev0JKZTfeBNG1JIB0mU4PtbmaTwJBuXzKZjrAFaLAljySnYbf5772SbGOI3TLhSdLRPffqfhz8SWkidZ4mSK0Zz4gkKU4Yp3qTUSykXokaoFYkFt8ZX2Itc1oDpMh5HAnJqcRntfOF4JqTlOLOVypjwkkHoT9iugAH2JtOv40n15MZPvLfBV4YVsj15PPVuMTCQeiF4woLA-_Lv7OgRIP3wSA8rdDxZAuNvOA89Cvo6jaceRXpu_AsvKiw0WTmB7MSPO_1e4PRNlB_cuNd3QJ6i49O3w964_4LUkAuet6H0aAhTsB6_bOz8wuEapmJRJoJTrnG_yMMvxGJ1kubxrFIhMTDYcC13OCpYFxfXuKjvu9_gDzYu0LGVLp79dDRM0IrSgDDN7nSVP6dz7EnoxXTNNK5hPxAPeiTRgshAMafKRSMaBpDoWDHtDH4gT1OtEgYu1sljKxXWrxNLV13lc8RyiBSsoSSTiVV8ACimeD4mmyo7GCi8M7xUWh_xSO3j76_W9EkwW-8LPvrwVgpu9979b6RT9Vai6xmFruTbyTLfmd8R-UjvLnm0h4cYprCw7tx6dxn4ilXO_6BIJQmtzyGWlWd3cijyjie8rQeCHCYJtineK5lfAK1pEpLl9s_8bVYssgm3a7as20c4zchiaA5nE97MhxuGYW0XdYbn0KXWfvKbUiJ6R7VKQencND0uoRFlCuQCOvF7Sg4MtmHDfjYmeBWHpLHTFsOu3oFg5BLwtl_NleWqH7wCj4Ww5rpjeWym9eQkNTZgwT82vp5rwi-SAkzRR8LSQ-pxJqCHCfWm1vaLl-COIG_rVC22sfWoety53s49L9M3gYmC4yqlyzPO816xReSQGvlTnq2AmGBX_dqNyaazImizr3p1CmVioEKcZuvmpo32j-SPHHtY1c211EuoXwHGCsYBniCm023NgPOEqo2EEF6AAV9hHV3nitLYnfKDHiuDjBOxHJJpRsguwTLsRRKdaNca5gtM6kufY32MocSpS7-Yg0MntrwCM_c_jcM8BfCHovk2yXY35AIYjCGwWnQB0Mo9jEGNVXIiTd0zHfevSw1ce_UiRxCdcV-YbFV5Baz6tzq10fcOY4SBnEf44UTqU5B8vKiO69dhOidIiJ75-S-9p_9AJaFeLWgVpZasLrStFwBAWhDjLRAbBbt3m-_PgzoJtmE_o9bFaO9f89NrH1iId6o1OKTlqtOZ0-aHZZg9cHzcHJp5hIS20E7wtpCsHenkceJagtBCTZbgpy22Vmo0Qqy4z4Kyp7Zu1fMTgvqRrMFjOctQKEnLagVqxbMiQj6BB8I8Ln-EwsgvqU=)

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
