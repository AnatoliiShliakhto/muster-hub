//! # Event Bus
//!
//! A high-performance, type-safe, asynchronous event bus designed for
//! vertical slice architectures.
//!
//! ## Overview
//!
//! Provides a centralized `EventBus` with multiple channel kinds (`broadcast`, `mpsc`, `watch`)
//! to connect decoupled components. Uses `tokio` primitives with minimal overhead.
//!
//! ## Features
//!
//! * **Type-Safe**: Events are identified by their Rust type.
//! * **Channel choice**: Broadcast (fan-out), MPSC (queue), Watch (the latest value).
//! * **High Performance**: `FxHashMap` + `parking_lot::RwLock`.
//! * **Async Ready**: Built on top of `tokio`.
//! * **Vertical Slice Friendly**: Share a single bus across slices.
//!
//! # Example
//!
//! ```rust
//! use mhub_event_bus::{EventBus, EventReceiverExt, ChannelKind, EventBusError};
//!
//! #[derive(Clone, Debug, PartialEq)]
//! struct UserCreated { id: u64 }
//!
//! #[tokio::main]
//! async fn main() -> Result<(), EventBusError> {
//!     let bus = EventBus::new();
//!
//!     // Default broadcast channel.
//!     let mut rx = bus.subscribe::<UserCreated>()?;
//!     bus.publish(UserCreated { id: 42 })?;
//!
//!     if let Ok(event) = rx.recv().await {
//!         assert_eq!(event.id, 42);
//!     }
//!     Ok(())
//! }
//! ```

mod bus;
mod error;
mod receiver;

pub use bus::{ChannelKind, Event, EventBus};
pub use error::{EventBusError, EventBusErrorExt};
pub use receiver::EventReceiverExt;
