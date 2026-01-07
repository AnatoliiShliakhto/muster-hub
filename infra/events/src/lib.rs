//! # Event Bus
//!
//! A high-performance, type-safe, asynchronous event bus designed for
//! vertical slice architectures.
//!
//! ## Overview
//!
//! This crate provides a centralized `EventBus` that allows different parts of
//! an application to communicate without direct coupling. It uses `tokio::sync::broadcast`
//! under the hood for efficient one-to-many communication.
//!
//! ## Features
//!
//! - **Type-Safe**: Events are identified by their Rust type.
//! - **High Performance**: Uses `FxHashMap` and `parking_lot::RwLock` for minimal overhead.
//! - **Async Ready**: Built on top of `tokio` for non-blocking I/O.
//! - **Vertical Slice Friendly**: Share a single bus across multiple domain slices.
//!
//! # Example
//!
//! ```rust
//! use event_bus::{EventBus, Result};
//!
//! #[derive(Clone, Debug, PartialEq)]
//! struct UserCreated { id: u64 }
//!
//! #[tokio::main]
//! async fn main() -> Result<()> {
//!     let bus = EventBus::new();
//!
//!     // 1. Subscribe to an event
//!     let mut rx = bus.subscribe::<UserCreated>()?;
//!
//!     // 2. Spawn a background handler (Simulating a Vertical Slice)
//!     tokio::spawn(async move {
//!         // Create a second subscription for the background task
//!         // Alternatively, you could pass the first 'rx' if not needed elsewhere
//!         while let Ok(event) = rx.recv().await {
//!             println!("Background processing for user: {}", event.id);
//!         }
//!     });
//!
//!     // 3. Publish an event
//!     bus.publish(UserCreated { id: 42 })?;
//!
//!     Ok(())
//! }
//! ```
//!
//! ### Handling Lagging Subscribers
//!
//! Because this bus uses broadcast channels, slow subscribers can "lag".
//! Professionals handle this by checking for `RecvError::Lagged`:
//!
//! ```rust
//! # use event_bus::{EventBus, Event};
//! # #[derive(Clone, Debug)] struct Ping;
//! # async fn doc() -> Result<(), tokio::sync::broadcast::error::RecvError> {
//! # let mut rx = EventBus::new().subscribe::<Ping>().unwrap();
//! match rx.recv().await {
//!     Ok(event) => { /* handle event */ },
//!     Err(tokio::sync::broadcast::error::RecvError::Lagged(skipped)) => {
//!         eprintln!("System under load! Skipped {} messages", skipped);
//!     }
//!     Err(tokio::sync::broadcast::error::RecvError::Closed) => { /* bus shutdown */ }
//! }
//! # Ok(())
//! # }
//! ```
mod error;

pub use error::{Error, Result};

use fxhash::FxHashMap;
use parking_lot::RwLock;
use std::any::{Any, TypeId};
use std::sync::Arc;
use tokio::sync::broadcast;
use tracing::trace;

/// A safe default for the broadcast channel buffer.
/// 128 is usually sufficient for domain events in a vertical slice.
const DEFAULT_CAPACITY: usize = 128;

/// Marker trait for types that can be sent across the [`EventBus`].
///
/// Any type that is `Send + Sync + Clone + 'static` automatically implements this trait.
pub trait Event: Any + Send + Sync + Clone + 'static {}
impl<T: Any + Send + Sync + Clone + 'static> Event for T {}

/// A high-performance, thread-safe Event Bus.
///
/// The `EventBus` manages multiple broadcast channels internally, indexed by
/// the [`TypeId`] of the event. It is designed to be cloned and shared
/// across tasks or threads.
#[derive(Clone, Default)]
pub struct EventBus {
    /// Inner state wrapped in an Arc and RwLock for concurrent access.
    channels: Arc<RwLock<FxHashMap<TypeId, Box<dyn Any + Send + Sync>>>>,
}

impl EventBus {
    /// Creates a new, empty `EventBus`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Subscribes to an event of type `T` with the default buffer capacity.
    ///
    /// # Errors
    ///
    /// Returns [`Error::TypeMismatch`] if a channel for `T` exists but the
    /// internal downcast fails.
    pub fn subscribe<T: Event>(&self) -> Result<broadcast::Receiver<T>> {
        self.subscribe_with_capacity::<T>(DEFAULT_CAPACITY)
    }

    /// Subscribes to an event of type `T` with a specific buffer capacity.
    ///
    /// **Note:** The capacity is only applied if this is the first time
    /// anyone is subscribing to or publishing this event type.
    ///
    /// # Errors
    ///
    /// Returns [`Error::TypeMismatch`] if the internal dynamic dispatch fails.
    pub fn subscribe_with_capacity<T: Event>(
        &self,
        capacity: usize,
    ) -> Result<broadcast::Receiver<T>> {
        let type_id = TypeId::of::<T>();
        let type_name = std::any::type_name::<T>();

        // 1. Optimized Path: Try to get an existing sender with a Read Lock
        {
            let read_guard = self.channels.read();
            if let Some(any_sender) = read_guard.get(&type_id) {
                return any_sender
                    .downcast_ref::<broadcast::Sender<T>>()
                    .map(|s| s.subscribe())
                    .ok_or(Error::TypeMismatch(type_name));
            }
        }

        // 2. Creation Path: Get Write Lock to initialize the channel
        let mut write_guard = self.channels.write();

        let sender = write_guard.entry(type_id).or_insert_with(|| {
            trace!(event = type_name, capacity, "Initializing new event channel");
            let (tx, _) = broadcast::channel::<T>(capacity);
            Box::new(tx)
        });

        sender
            .downcast_ref::<broadcast::Sender<T>>()
            .map(|s| s.subscribe())
            .ok_or(Error::TypeMismatch(type_name))
    }

    /// Publishes an event to all active subscribers.
    ///
    /// # Returns
    ///
    /// Returns the number of subscribers that were reached. If there are no
    /// active subscribers, it returns `Ok(0)`.
    ///
    /// # Errors
    ///
    /// Returns [`Error::TypeMismatch`] if the internal dynamic dispatch fails.
    pub fn publish<T: Event>(&self, event: T) -> Result<usize> {
        let type_id = TypeId::of::<T>();
        let read_guard = self.channels.read();

        if let Some(any_sender) = read_guard.get(&type_id) {
            let sender = any_sender
                .downcast_ref::<broadcast::Sender<T>>()
                .ok_or_else(|| Error::TypeMismatch(std::any::type_name::<T>()))?; // Moved type_name here

            match sender.send(event) {
                Ok(count) => {
                    trace!(
                        event = std::any::type_name::<T>(),
                        subscribers = count,
                        "Event dispatched"
                    );
                    Ok(count)
                },
                Err(_) => Ok(0),
            }
        } else {
            Ok(0)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Debug, PartialEq)]
    struct UserCreated {
        id: u64,
    }

    #[tokio::test]
    async fn test_event_flow() {
        let bus = EventBus::new();
        let mut rx = bus.subscribe::<UserCreated>().unwrap();

        let event = UserCreated { id: 42 };
        bus.publish(event.clone()).unwrap();

        let received = rx.recv().await.unwrap();
        assert_eq!(received, event);
    }
}
