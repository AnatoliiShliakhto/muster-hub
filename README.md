# 🛡️ MusterHub

![Rust](https://img.shields.io/badge/built_with-Rust-dca282.svg)
![License](https://img.shields.io/badge/license-MIT-blue.svg)
![Version](https://img.shields.io/badge/version-0.0.0-green.svg)

> **⚠️ Project Status: Early Development (WIP)**  
> This project is currently in its initial development phase. Core architectures are being established, and features are subject to frequent breaking changes. **Not ready for production use.**

**MusterHub** is a high-performance, portable orchestration platform designed for secure data collection and task management in environments where internet connectivity is unreliable or restricted.

Built entirely in **Rust**, MusterHub allows a central Desktop "Hub" to act as a localized server, enabling users to participate in exams, surveys, and data gathering via a local Wi-Fi mesh – no external cloud required.

---

## 🚀 Key Features

*   **Portable "Hub" architecture**: Turn any laptop into a local server. The Hub manages the database, serves the web interface, and handles real-time reporting.
*   **Offline-first connectivity**: Creates a local Wi-Fi hotspot. Clients connect via standard web browsers or the native Android application.
*   **Versatile task engine**: Create and deploy:
    *   **Exams & quizzes**: Secure testing with real-time monitoring.
    *   **Surveys & polls**: Anonymous or identified data collection.
    *   **Field data gathering**: Collect grades, logistics, or custom structured data.
*   **Secure data vault**: Localized storage using SurrealDB for high-speed, schema-flexible data management and reporting.
*   **WASM-powered frontend**: Shared logic between the desktop shell and web clients using Dioxus and WebAssembly.

---

## 🏗️ Architecture

The project is structured as a modern Rust workspace for maximum code reuse:

-   `assets/`: Static assets (HTML, CSS, JS, Images, Fonts).
-   `apps/`: Contains the primary entry points (Server, Desktop, Mobile, Web).
-   `crates/`: Core business logic and domain models.
-   `infra/`: Infrastructure implementations (Database, Event Bus, Logger etc.).
-   `xtask/`: Custom automation scripts for building and deployment.

### How it Works
1.  **Orchestrate**: An administrator creates tasks on the MusterHub Desktop App.
2.  **Broadcast**: The app initializes a local network listener.
3.  **Engage**: Participants connect to the local IP; tasks are served via WASM.
4.  **Consolidate**: Data is streamed back to the Hub, validated, and stored in a local encrypted DB.
5.  **Report**: Generate reports immediately after the session ends.

---

## 🛠️ Tech Stack

-   **Language**: [Rust](https://www.rust-lang.org/) (Edition 2024)
-   **Frontend**: [Dioxus](https://dioxuslabs.com/) (Fullstack & WASM)
-   **Backend**: [Axum](https://github.com/tokio-rs/axum) (High-performance async web framework)
-   **Database**: [SurrealDB](https://surrealdb.com/) (Embedded multi-model database)
-   **Async runtime**: [Tokio](https://tokio.rs/)

---

## 🚦 Getting Started

### Prerequisites
-   Rust (latest stable)

### Development
1.  Clone the repository:
    ```bash
    git clone https://github.com/AnatoliiShliakhto/mhub-hub.git
    cd mhub-hub
    ```
2.  Run the Desktop Hub:
    ```bash
    cargo xtask run shell
    ```

---

## 🧪 Testing & CI

We maintain a "Strict-Rust" policy. Our CI (GitHub Actions) ensures:
-   Zero-warning compilation (`-D warnings`).
-   Full workspace unit and integration testing.
-   Clippy linting for professional-grade code quality.
-   WASM compatibility checks.

```bash
cargo xtask test --all
```