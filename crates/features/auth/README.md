
```text
crates/identity/
├── src/
│   ├── domain/          # 1. THE CORE: Pure business logic & types (No dependencies)
│   │   ├── models/      # User, Session, Role, Permission
│   │   ├── services/    # Logic: "How to calculate if a password is expired"
│   │   └── repository.rs # TRAITS: Definitions of how we save/load (not the impl)
│   │
│   ├── infrastructure/  # 2. THE PLUMBING: Implementation of traits
│   │   └── surreal/     # SurrealDB specific queries and schema
│   │
│   ├── api/             # 3. THE SERVER: Axum handlers & DTOs (Data Transfer Objects)
│   │   ├── handlers/    # login_handler, register_handler
│   │   └── routes.rs    # Router definition for this feature
│   │
│   ├── ui/              # 4. THE CLIENT: Dioxus components (Shared/WASM)
│   │   ├── components/  # LoginForm, UserProfile, PermissionGuard
│   │   └── views/       # LoginView, DashboardView
│   │
│   ├── lib.rs           # Crate entry point & public re-exports
│   └── error.rs         # Feature-specific error enum
└── Cargo.toml
```

Detailed Breakdown of the Layers
1. domain/ (The "Source of Truth")
   This is the most important folder. It contains pure Rust.
   Why: You should be able to test your identity logic without a database or a web server.
   Repository Traits: Define a trait like pub trait UserRepository { fn find_by_id(...); }. This allows you to swap SurrealDB for a Mock during tests.
2. infrastructure/ (The "Worker")
   This is where you implement the UserRepository using mhub-db.
   Why: If you decide to move from SurrealDB to PostgreSQL in two years, you only change this one folder. The rest of the "huge amount of code" remains untouched.
3. api/ (The "Bridge")
   These are your Axum handlers. They take a request, call the domain/services, and return JSON.
   Pro-Tip: Keep these handlers thin. They should only handle HTTP concerns (status codes, cookies). The actual logic stays in domain/.
4. ui/ (The "Face")
   Since you are using Dioxus, this folder contains your .rsx! macros.
   WASM Compatibility: Ensure this folder doesn't try to use server-only libraries (like tokio::fs).
   Shared Logic: The UI uses the models from domain/models to ensure the "User" on the screen is exactly the same "User" in the database.

```rust
// crates/identity/src/lib.rs

pub mod domain;
pub mod error;

#[cfg(feature = "server")]
pub mod api;

#[cfg(feature = "server")]
pub mod infrastructure;

#[cfg(feature = "ui")]
pub mod ui;

// Re-export common types for a better "Pro" DX (Developer Experience)
pub use domain::models::*;
pub use error::{Error, Result};


```