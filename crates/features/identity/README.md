# mhub-identity 🔐

Identity feature slice for JWT + DPoP authentication and Axum middleware.
Provides an in-memory session store (Moka) and DPoP nonce/replay protection.

## Status

- Initializes an `Identity` slice with JWT/DPoP auth services and in-memory session cache.
- Feature-gated: `server` / `client` propagate to `mhub-kernel` for Axum/Dioxus contexts.