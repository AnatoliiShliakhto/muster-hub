# MusterHub Architecture

## Vertical Slice Architecture (VSA)

![Architecture](https://kroki.io/plantuml/svg/eNqNVttu2zgQfedXcB1g031wmrZO7BRGUFmXNtgkNpxuX4KgoCXaIiyRAkmlMbr59x2Skqw41sZ-GvLwzH1G_qI0kbrMM6TWjBdEkhxnjFO9KSgWUqeiBaiUJOIX4yusZUlbgBQlT2IhOZX4rHW_FFxzklPc-0FlQjjpIfRHQpdgAHuz2U9_ej2d46PgQ-hFUYNcT79e-TUWjSIv8hssCr3v_8zDGg0_hqPotEH96RaaeOF5FDTQ1W009xqlH6Kz6KLBJtN5EM5r8HwwHIwmW0eD6Y13dQvoLT46_Tga-MNXSgG5GHifJqM9fgI2GJ6dnV8g1MpMLPJCcMo1_o0w_CYkXq9sGn2RCYnH45BrucEzwbi-vMRHwyD4BHmwb4VMqHTv2q6jZ4RSSgDDN6XSVH4rF9iTcco0jXUpIT9QD_qk0VIIgPFXCgUjmiZQKDgxbQj_Yo8TLTLG7tKMkXWqxfvcquun5QKhAjwlKyjpTFIFARDNBMfXZENlDxOFX1wfRfZXBbkN-v4upVmG33lF8deDYSl73ol6lxRQtdaiaNESd_OTFMVb5DsqHyHmlkl7cQg1h8D7SW08YOKpVC_sg4JImtzyBGrVdPZePar246nM246ADtMEuyqeWxmfQi2p0tLl9k98LVYstkm3Une2jWH8LiIxNIezaW_G461GIW2XDfxT6DLLb8xGlJjuUb16cCoD-6LLWEy5ghVhrbgTBUMm-3AAGy8muFMPS0BmemPVkFKnb-h43vE5EDlhJke-kPQQx9cUtldmzTnRNsUKZhnsbfdKJz-xBl1TONvjcfB9-j40YTCqXmt5flHbK76UBCpRukndzpMF_r-0_YRosiCKOvOmsDMqFYOh5TZfreW3l_9IykxbspVsruNSQv4PICvoHQjBtbKTzTywjKoNeJAfoII-gtxflMoqsSdl5qFUB5AzsVpR6frNisD0pVCqH5daQyuaxnbp28uXJZQod_5XMmjw1IbHeO7Ob2iAjcseq-RbEfg3JAYfDDE8DYdAhGIfY1g-CrldBx3zi_cv6xWyc-t2AkLtBfeKsV1gHbTm3o77Z9w7jjMGfh_jpZvpXqXk9UN333oI3rsFguybk_vWJ-4BmNWsd6BmhB3Uv99-UA3ips2o_9tJ1fjtvnNTZd2o9hGq18tJx1O7N07226uw5gv-cHJpJgdCt6G6r0WdnZ13VZd0oK4JO8Bk0QFUk9OB2rHswNy4oC_w5YD_cf8Brckm2A==)

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