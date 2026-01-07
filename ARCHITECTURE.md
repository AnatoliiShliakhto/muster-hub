# MusterHub Architecture

## Vertical Slice Architecture (VSA)

### Structure

- `crates/features/*` - Independent vertical slices (identity, exam, survey)
- `crates/shared/kernel` - Shared business logic and utilities
- `crates/shared/domain` - Pure domain models (no dependencies)
- `apps/*` - Application entry points that compose features
- `infra/*` - Infrastructure abstractions (DB, events, logging)

### Feature Anatomy

Each feature is fully self-contained:

```text
features/{name}/
├── src/
│   ├── domain/         # Feature-specific domain models
│   ├── server/         # HTTP handlers (feature = "server")
│   ├── client/         # Dioxus UI Components (feature = "client")
│   ├── services/       # Business logic
│   ├── repository/     # Data access layer
│   └── error.rs        # Feature-specific errors
└── tests/
    ├── fixtures/       # Test fixtures
    │    └── mod.rs
    └── integration.rs  # Integration tests
```

### Dependency Rules

1. Features can depend on `shared/*` and `infra/*`
2. Features MUST NOT depend on other features
3. Features communicate via events (use `infra/events`)
4. `domain` has ZERO external dependencies