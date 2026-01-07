# MusterHub Architecture

## Vertical Slice Architecture (VSA)

![Architecture](https://kroki.io/plantuml/svg/eNqdVl1z2jgUfdev0JKZTfeBNG1JIB0mU4PtbmaTwJBuXzKZjrAFaLAljySnYbf5772SbGOI3TLhSdLRPffqfhz8SWkidZ4mSK0Zz4gkKU4Yp3qTUSykXokaoFYkFt8ZX2Itc1oDpMh5HAnJqcRntfOF4JqTlOLOVypjwkkHoT9iugAH2JtOv40n15MZPvLfBV4YVsj15PPVuMTCQeiF4woLA-_Lv7OgRIP3wSA8rdDxZAuNvOA89Cvo6jaceRXpu_AsvKiw0WTmB7MSPO_1e4PRNlB_cuNd3QJ6i49O3w964_4LUkAuet6H0aAhTsB6_bOz8wuEapmJRJoJTrnG_yMMvxGJ1kubxrFIhMTDYcC13OCpYFxfXuKjvu9_gDzYu0LGVLp79dDRM0IrSgDDN7nSVP6dz7EnoxXTNNK5hPxAPeiTRgshAMafKRSMaBpDoWDHtDH4gT1OtEgYu1sljKxXWrxNLV13lc8RyiBSsoSSTiVV8ACimeD4mmyo7GCi8M7xUWh_xSO3j76_W9EkwW-8LPvrwVgpu9979b6RT9Vai6xmFruTbyTLfmd8R-UjvLnm0h4cYprCw7tx6dxn4ilXO_6BIJQmtzyGWlWd3cijyjie8rQeCHCYJtineK5lfAK1pEpLl9s_8bVYssgm3a7as20c4zchiaA5nE97MhxuGYW0XdYbn0KXWfvKbUiJ6R7VKQencND0uoRFlCuQCOvF7Sg4MtmHDfjYmeBWHpLHTFsOu3oFg5BLwtl_NleWqH7wCj4Ww5rpjeWym9eQkNTZgwT82vp5rwi-SAkzRR8LSQ-pxJqCHCfWm1vaLl-COIG_rVC22sfWoety53s49L9M3gYmC4yqlyzPO816xReSQGvlTnq2AmGBX_dqNyaazImizr3p1CmVioEKcZuvmpo32j-SPHHtY1c211EuoXwHGCsYBniCm023NgPOEqo2EEF6AAV9hHV3nitLYnfKDHiuDjBOxHJJpRsguwTLsRRKdaNca5gtM6kufY32MocSpS7-Yg0MntrwCM_c_jcM8BfCHovk2yXY35AIYjCGwWnQB0Mo9jEGNVXIiTd0zHfevSw1ce_UiRxCdcV-YbFV5Baz6tzq10fcOY4SBnEf44UTqU5B8vKiO69dhOidIiJ75-S-9p_9AJaFeLWgVpZasLrStFwBAWhDjLRAbBbt3m-_PgzoJtmE_o9bFaO9f89NrH1iId6o1OKTlqtOZ0-aHZZg9cHzcHJp5hIS20E7wtpCsHenkceJagtBCTZbgpy22Vmo0Qqy4z4Kyp7Zu1fMTgvqRrMFjOctQKEnLagVqxbMiQj6BB8I8Ln-EwsgvqU=)

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