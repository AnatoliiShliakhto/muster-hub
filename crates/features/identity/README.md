# mhub-identity 🔐

Identity feature slice for JWT + DPop authentication and Axum middleware.
Provides an in-memory session store (Moka) and DPop nonce/replay protection.

## Status

- Initializes an `Identity` slice with JWT/DPop auth services and in-memory session cache.
- Feature-gated: `server` / `client` propagate to `mhub-kernel` for Axum/Dioxus contexts.