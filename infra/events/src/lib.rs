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
//! * **Type-Safe**: Events are identified by their Rust type.
//! * **High Performance**: Uses `FxHashMap` and `parking_lot::RwLock` for minimal overhead.
//! * **Async Ready**: Built on top of `tokio` for non-blocking I/O.
//! * **Vertical Slice Friendly**: Share a single bus across multiple domain slices.
//!
//! # Example
//!
//! ```rust
//! use mhub_event_bus::{EventBus, EventReceiverExt, Result};
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
//!         while let Some(event) = rx.recv_event().await {
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

mod error;

pub use error::{EventBusError, EventBusErrorExt, Result};

use fxhash::FxHashMap;
use parking_lot::RwLock;
use std::any::{Any, TypeId};
use std::sync::Arc;
use tokio::sync::broadcast;
use tracing::{trace, warn};

/// A safe default for the broadcast channel buffer.
/// 128 is usually enough for domain events in a vertical slice.
const DEFAULT_CAPACITY: usize = 128;

/// Marker trait for types that can be sent across the [`EventBus`].
///
/// Any type that is `Send + Sync + 'static` automatically implements this trait.
pub trait Event: Any + Send + Sync + 'static {}
impl<T: Any + Send + Sync + 'static> Event for T {}

/// A high-performance, thread-safe Event Bus.
///
/// The `EventBus` manages multiple broadcast channels internally, indexed by
/// the [`TypeId`] of the event. It is designed to be cloned and shared
/// across tasks or threads.
#[derive(Debug, Clone, Default)]
pub struct EventBus {
    /// Inner state wrapped in an `Arc` and `RwLock` for concurrent access.
    channels: Arc<RwLock<FxHashMap<TypeId, Box<dyn Any + Send + Sync>>>>,
}

impl EventBus {
    /// Creates a new, empty `EventBus`.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Subscribes to an event of type `T` with the default buffer capacity.
    ///
    /// # Errors
    ///
    /// Returns [`EventBusError::TypeMismatch`] if a channel for `T` exists but the
    /// internal downcast fails.
    pub fn subscribe<T: Event>(&self) -> Result<broadcast::Receiver<Arc<T>>> {
        self.subscribe_with_capacity::<T>(DEFAULT_CAPACITY)
    }

    /// Subscribes to an event of type `T` with a specific buffer capacity.
    ///
    /// **Note:** The capacity is only applied if this is the first time
    /// anyone is subscribing to or publishing this event type.
    ///
    /// # Errors
    ///
    /// Returns [`EventBusError::TypeMismatch`] if the internal dynamic dispatch fails.
    pub fn subscribe_with_capacity<T: Event>(
        &self,
        capacity: usize,
    ) -> Result<broadcast::Receiver<Arc<T>>> {
        let id = TypeId::of::<T>();

        {
            let channels = self.channels.read();
            if let Some(sender) = channels.get(&id) {
                return sender
                    .downcast_ref::<broadcast::Sender<Arc<T>>>()
                    .map(broadcast::Sender::subscribe)
                    .ok_or_else(|| EventBusError::TypeMismatch {
                        message: std::any::type_name::<T>().into(),
                        context: Some("Unexpected event type".into()),
                    });
            }
        }

        let mut channels = self.channels.write();
        let sender = channels.entry(id).or_insert_with(|| {
            trace!(event = std::any::type_name::<T>(), capacity, "Initializing new event channel");
            let (tx, _) = broadcast::channel::<Arc<T>>(capacity);
            Box::new(tx)
        });

        let result = sender
            .downcast_ref::<broadcast::Sender<Arc<T>>>()
            .map(broadcast::Sender::subscribe)
            .ok_or_else(|| EventBusError::TypeMismatch {
                message: std::any::type_name::<T>().into(),
                context: Some("Unexpected event type".into()),
            });

        drop(channels);
        result
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
    /// Returns [`EventBusError::TypeMismatch`] if the internal dynamic dispatch fails.
    pub fn publish<T: Event>(&self, event: T) -> Result<usize> {
        let id = TypeId::of::<T>();
        let channels = self.channels.read();

        if let Some(sender) = channels.get(&id) {
            let sender = sender.downcast_ref::<broadcast::Sender<Arc<T>>>().ok_or_else(|| {
                EventBusError::TypeMismatch {
                    message: std::any::type_name::<T>().into(),
                    context: Some("Unexpected event type".into()),
                }
            })?;

            sender.send(Arc::new(event)).map_or_else(
                |_| {
                    trace!(
                        event = std::any::type_name::<T>(),
                        "Event dropped: no active subscribers"
                    );
                    Ok(0)
                },
                |count| {
                    trace!(event = std::any::type_name::<T>(), count, "Event dispatched");
                    Ok(count)
                },
            )
        } else {
            Ok(0)
        }
    }
}

// --- Event Receiver Extensions  ---

/// An extension trait for [`broadcast::Receiver`] to provide
/// a more ergonomic and resilient event-processing API.
pub trait EventReceiverExt<T> {
    /// Receives the next event from the bus, automatically handling "Lagged" errors.
    ///
    /// ### Resilient Processing
    /// In a broadcast-based system, a subscriber that processes messages slower than
    /// the publisher will eventually "lag," causing the underlying channel to drop
    /// older messages to make room for new ones.
    ///
    /// Standard receivers return an `Err(Lagged)` in this scenario. This method
    /// handles that error by:
    /// 1. Logging a warning with the count of skipped messages via `tracing`.
    /// 2. Automatically resuming to the next available fresh message.
    ///
    /// ### Return Value
    /// * `Some(T)`: A successfully received event.
    /// * `None`: The event bus has been shut down or all senders have been dropped.
    ///
    /// ### Examples
    ///
    /// ```rust
    /// use mhub_event_bus::{EventBus, EventReceiverExt};
    ///
    /// #[derive(Clone, Debug, PartialEq)]
    /// struct UserCreated { id: u64 }
    ///
    /// # async fn doc(bus: EventBus) {
    /// let mut rx = bus.subscribe::<UserCreated>().unwrap();
    ///
    /// while let Some(event) = rx.recv_event().await {
    ///     println!("Received: {:?}", event);
    /// }
    /// # }
    /// ```
    fn recv_event(&mut self) -> impl Future<Output = Option<Arc<T>>> + Send;
}

impl<T: Event> EventReceiverExt<T> for broadcast::Receiver<Arc<T>> {
    async fn recv_event(&mut self) -> Option<Arc<T>> {
        loop {
            match self.recv().await {
                Ok(event) => return Some(event),
                Err(broadcast::error::RecvError::Lagged(n)) => {
                    warn!(
                        event = std::any::type_name::<T>(),
                        skipped = n,
                        "EventBus receiver lagged; continuing from latest message"
                    );
                },
                Err(broadcast::error::RecvError::Closed) => return None,
            }
        }
    }
}
