# MusterHub Architecture

## Vertical Slice Architecture (VSA)

![Architecture](https://kroki.io/plantuml/svg/eNqNVt1v2zYQf9dfcXOAZXuwt6Rdmw1FUNkyuwBZbNjp9jAMBS3RFmGJFEjKgbf1f9-R1AftKkv94uPd_Y73Tb3XhipTl0V0CePxGGZSbPmuVtRwKSwn0nsuKqpoCQUXzBwrBlKZXAYCndNMPnGxgy0tNAskStYiS6USTMFNwN9KYQQtGYx-Zyqjgo6izoFCKkjYlgtufdDwnTuwDJ44XlsbMDmDCzAS6EHyDDJZbwo2zqnO0YXvndPfZA4D8XL5aba4X6wguZrHhHQCEs_iZN7I3ibJKzLrZfP48eOqFZIbEgfC2aKTTOP5G5J0krsHsopb0BX5ifzcW1x8fEjix7vFQysnqHHdyaeLVTJfNbI3r9--vpm2-VibY8H0WSUqmu7pjsE_EeBviqedy7RP3h85N8xLpMqY8tyL8JLocxSYS2VZScGEaQ0-Cxu47t27uTDqCEvJhbm9RYBPy6AqUVh4JjKrN_-RXJP5sJ5lnKqhwzmj6Bb8VmvD1K_1BmKV2lBTUyvMMUbCnqTaR1spUQE-MOw6arBxthjHB24s5F-IBTWy4HydF5zucyN_KJ3BcV5v2qTf0yNTPulRm-vR1QSWimlMkx8OpzQCqk_ZPoV9Sv9c56wo_rJ62lJwlrBz9YTpvZEVxFXlQJk_f6JV9RJ0zdQBA2-R2h2fBZ6jm4vCSyGs2Lm-Nx_cBEHdbL26zF1P4F7ueArfwgJLxrTx-8Ulz0u-yFqJ5XC2LWH9oClW3zZEOLp9IN1lhFHbD9oZbw-N_bM7aJ1x4y5xFJoOJ38IIdWOCv63894BQ8ZX4HmGf9wcHdYdvgZES6-PozqgjZk-ScBMKuaCt8Rw4HuGG9n3pCfhot9rQ4BMlpT7iD2JgBmZkqvGg7DaryZwJ7aKYpVrN5rOGcdqvOlUlzhlHIdPpAx7YybLshY8bXpj0POMGrqhmnlXNnARbN0hfXbAv_Gm1g7gTvpFkDZSoX--tT39JebzaShrltYKK-vjqHC3YFDPBHGgdeH7zlEv-lPwlAmNj5vD-NPLHhG7T10uaeGHoWMMu8WUkn6eHYW9dvZsDYFUjQ1d-lw1NAL9mzkUitztmqXhyf9Txm3PD02pHYmvwRxfA9J3XbOwV6xwgeEnQNXsbb9sx-PbdpkFHL-vonC1nij2izPQbtVhMrn1W-kXGF2mBUdfL0cN6FToeSiMHMsaazdSz7FjGnWLquM0g-n8ciNnY31kaW4npEBfK1y0ODkcbfkdaq92Y4aKJ09SL_CF9bHahEZtyQKW78mAEb3Hm_AT8T9x6zbc)

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