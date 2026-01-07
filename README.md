# MusterHub 🛡️

![Rust](https://img.shields.io/badge/built_with-Rust-dca282.svg)
![No Unsafe](https://img.shields.io/badge/unsafe-forbidden-success.svg)
![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)
![Version](https://img.shields.io/badge/version-0.0.0-green.svg)
![GitHub Workflow Status](https://img.shields.io/github/actions/workflow/status/AnatoliiShliakhto/muster-hub/ci.yml?branch=dev)

**MusterHub** is a high-performance, enterprise-grade ecosystem designed for comprehensive training management and
administration within complex organizational hierarchies. Far more than a simple tracking tool, it is an active
operational environment where administrators, instructors, and cadets manage the entire educational lifecycle in
real-time.

## Status

> **⚠️ Project Status: Early Development (WIP)**  
> This project is currently in its initial development phase. Core architectures are being
> established, and features are subject to frequent breaking changes. **Not ready for production
> use.**

## Core Capabilities & Architecture

* **Active Academic Workflow:** A complete suite for instructors to conduct exams, log grades, manage credits, and
  generate deep analytical performance reports.
* **Advanced Scheduling & Dispatch:** A powerful engine for building conflict-free timetables, coordinating instructors,
  student groups, and facility resources.
* **High-Load Scalability:** Built with Rust and Axum, the system handles massive throughput using SurrealDB for
  instantaneous graph-based queries across deep organizational structures.
* **Observability & Intelligence:** Full system transparency via OpenTelemetry (OTLP) for real-time performance
  monitoring and enterprise-level auditing.
* **Offline-First Mobility:** Purpose-built mobile applications for field operations (attendance, grading, and
  assessments) with seamless delta-sync capabilities.

## Security & Access (Fortress-First)

Security is woven into the core through a multi-layered access model and industry-leading cryptographic standards:

* **Node-Based RBAC & Matrix Access:** Dynamic Role-Based Access Control where permissions are scoped to specific
  organizational nodes. A matrix-rights system ensures users interact only with the modules and data permitted by their
  specific department and role.
* **Authentication:** Multi-layered identity verification supporting OAuth2 flows, TOTP (Time-based One-Time Password),
  and FIDO2/WebAuthn for hardware-level security.
* **Cryptographic Excellence:** Data protection via AES-256-GCM and ChaCha20.
* **Hardened Identity:** Advanced session security using Argon2 hashing and JWT + DPoP digitally signed using Ed25519 to
  prevent token theft, combined with the Zeroize pattern for proactive memory hygiene.
* **Immutable Audit:** A tamper-proof audit pipeline that captures every state change—from grade entries to
  permission shifts—ensuring total institutional accountability.

## Tech Stack

* **Language:** Pure Rust – Leveraged for memory safety, high concurrency, and zero-cost abstractions, ensuring the core
  engine is both fast and crash-proof.
* **API Layer:** Axum – A high-performance, asynchronous web framework built on the tokio ecosystem, providing the
  backbone for the enterprise-grade API.
* **Frontend:** Dioxus – A modern, full-stack Rust UI library used to build the Web dashboard, Desktop admin tools, and
  Mobile field apps from a single codebase.
* **Database:** SurrealDB – A multimodel (Document, Graph, and Relational) database. It is uniquely suited for
  MusterHub’s hierarchical RBAC, allowing complex path-based queries through organizational nodes without the
  performance penalties of traditional SQL joins.
* **Observability & Telemetry:** Enterprise-Grade Tracing – Powered by the tracing ecosystem and OpenTelemetry (OTLP).
    * Uses opentelemetry-sdk and opentelemetry-otlp to export distributed traces and metrics to centralized collectors.
    * Provides deep visibility into the system's "internal state" for debugging complex scheduling logic and auditing
      security events.
* **Profiling:** Integrated tokio-console for real-time task debugging and dhat for precise heap memory tracking.

## Audience & Prereqs

- Rust toolchain from `rust-toolchain.toml`.
- Docker (for `cargo xtask dev`).
- Node.js (for `tailwindcss`).

## Workspace Layout

```text
.
├── apps/               # Entrypoint & Binaries
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

![Architecture](https://kroki.io/plantuml/svg/eNqNVt1v2zYQf9dfcXOAZXuwt6Rdmw1FUNkyuwBZbNjp9jAMBS3RFmGJFEjKgbf1f9-R1AftKkv94uPd_Y73Tb3XhipTl0V0CePxGGZSbPmuVtRwKSwn0nsuKqpoCQUXzBwrBlKZXAYCndNMPnGxgy0tNAskStYiS6USTMFNwN9KYQQtGYx-Zyqjgo6izoFCKkjYlgtufdDwnTuwDJ44XlsbMDmDCzAS6EHyDDJZbwo2zqnO0YXvndPfZA4D8XL5aba4X6wguZrHhHQCEs_iZN7I3ibJKzLrZfP48eOqFZIbEgfC2aKTTOP5G5J0krsHsopb0BX5ifzcW1x8fEjix7vFQysnqHHdyaeLVTJfNbI3r9--vpm2-VibY8H0WSUqmu7pjsE_EeBviqedy7RP3h85N8xLpMqY8tyL8JLocxSYS2VZScGEaQ0-Cxu47t27uTDqCEvJhbm9RYBPy6AqUVh4JjKrN_-RXJP5sJ5lnKqhwzmj6Bb8VmvD1K_1BmKV2lBTUyvMMUbCnqTaR1spUQE-MOw6arBxthjHB24s5F-IBTWy4HydF5zucyN_KJ3BcV5v2qTf0yNTPulRm-vR1QSWimlMkx8OpzQCqk_ZPoV9Sv9c56wo_rJ62lJwlrBz9YTpvZEVxFXlQJk_f6JV9RJ0zdQBA2-R2h2fBZ6jm4vCSyGs2Lm-Nx_cBEHdbL26zF1P4F7ueArfwgJLxrTx-8Ulz0u-yFqJ5XC2LWH9oClW3zZEOLp9IN1lhFHbD9oZbw-N_bM7aJ1x4y5xFJoOJ38IIdWOCv63894BQ8ZX4HmGf9wcHdYdvgZES6-PozqgjZk-ScBMKuaCt8Rw4HuGG9n3pCfhot9rQ4BMlpT7iD2JgBmZkqvGg7DaryZwJ7aKYpVrN5rOGcdqvOlUlzhlHIdPpAx7YybLshY8bXpj0POMGrqhmnlXNnARbN0hfXbAv_Gm1g7gTvpFkDZSoX--tT39JebzaShrltYKK-vjqHC3YFDPBHGgdeH7zlEv-lPwlAmNj5vD-NPLHhG7T10uaeGHoWMMu8WUkn6eHYW9dvZsDYFUjQ1d-lw1NAL9mzkUitztmqXhyf9Txm3PD02pHYmvwRxfA9J3XbOwV6xwgeEnQNXsbb9sx-PbdpkFHL-vonC1nij2izPQbtVhMrn1W-kXGF2mBUdfL0cN6FToeSiMHMsaazdSz7FjGnWLquM0g-n8ciNnY31kaW4npEBfK1y0ODkcbfkdaq92Y4aKJ09SL_CF9bHahEZtyQKW78mAEb3Hm_AT8T9x6zbc)

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
