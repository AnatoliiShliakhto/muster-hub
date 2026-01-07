use crate::error::EventBusError;
use fxhash::FxHashMap;
use parking_lot::RwLock;
use std::any::{Any, TypeId};
use std::sync::Arc;
use tokio::sync::{broadcast, mpsc, watch};
use tracing::{trace, warn};

/// A safe default for channel buffers.
/// 128 is usually enough for domain events in a vertical slice.
const DEFAULT_CAPACITY: usize = 128;
const MIN_CAPACITY: usize = 1;

/// Supported channel kinds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChannelKind {
    /// Broadcast (fan-out) semantics.
    Broadcast { capacity: usize },
    /// MPSC (queue) semantics.
    Mpsc { capacity: usize },
    /// Watch (latest-value) semantics.
    Watch,
}

/// Marker trait for types that can be sent across the [`EventBus`].
///
/// Any type that is `Send + Sync + 'static` automatically implements this trait.
pub trait Event: Any + Send + Sync + 'static {}
impl<T: Any + Send + Sync + 'static> Event for T {}

#[derive(Debug)]
struct ChannelState {
    kind: ChannelKind,
    sender: Box<dyn Any + Send + Sync>,
}

#[derive(Debug)]
struct MpscChannel<T> {
    sender: mpsc::Sender<Arc<T>>,
    receiver: Option<mpsc::Receiver<Arc<T>>>,
    taken: bool,
}

#[derive(Debug)]
enum ChannelHandle<T> {
    Broadcast(broadcast::Sender<Arc<T>>),
    Watch(watch::Sender<Arc<T>>),
}

impl<T: Event> ChannelHandle<T> {
    fn from_state(kind: ChannelKind, state: &ChannelState) -> Result<Self, EventBusError> {
        match kind {
            ChannelKind::Broadcast { .. } => {
                let sender =
                    state.sender.downcast_ref::<broadcast::Sender<Arc<T>>>().ok_or_else(|| {
                        EventBusError::TypeMismatch {
                            message: std::any::type_name::<T>().into(),
                            context: Some("Unexpected event type".into()),
                        }
                    })?;
                Ok(Self::Broadcast(sender.clone()))
            },
            ChannelKind::Watch => {
                let sender =
                    state.sender.downcast_ref::<watch::Sender<Arc<T>>>().ok_or_else(|| {
                        EventBusError::TypeMismatch {
                            message: std::any::type_name::<T>().into(),
                            context: Some("Unexpected event type".into()),
                        }
                    })?;
                Ok(Self::Watch(sender.clone()))
            },
            ChannelKind::Mpsc { .. } => unreachable!("MPSC channels are handled separately"),
        }
    }
}

/// A high-performance, thread-safe Event Bus.
///
/// Manages channels indexed by [`TypeId`] of the event.
#[derive(Debug, Clone, Default)]
pub struct EventBus {
    channels: Arc<RwLock<FxHashMap<TypeId, ChannelState>>>,
}

impl EventBus {
    /// Creates a new, empty `EventBus`.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Subscribes to an event of type `T` using broadcast with default capacity.
    ///
    /// # Errors
    /// Returns [`EventBusError::ChannelKindMismatch`] if a different channel kind
    /// was already registered for `T`.
    ///
    /// # Examples
    /// ```rust
    /// use mhub_event_bus::{EventBus, EventReceiverExt};
    ///
    /// #[derive(Clone, Debug, PartialEq)]
    /// struct UserCreated(u64);
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), mhub_event_bus::EventBusError> {
    /// let bus = EventBus::new();
    /// let mut rx = bus.subscribe::<UserCreated>()?;
    /// bus.publish(UserCreated(1))?;
    /// assert_eq!(rx.recv().await.unwrap().0, 1);
    /// # Ok(())
    /// # }
    /// ```
    pub fn subscribe<T: Event>(&self) -> Result<broadcast::Receiver<Arc<T>>, EventBusError> {
        self.subscribe_with_capacity::<T>(DEFAULT_CAPACITY)
    }

    /// Subscribes to an event of type `T` with a specific broadcast buffer capacity.
    ///
    /// # Errors
    /// Returns [`EventBusError::ChannelKindMismatch`] if a different channel kind
    /// was already registered for `T`, or [`EventBusError::InvalidCapacity`] if
    /// `capacity` is zero.
    ///
    /// # Examples
    /// ```rust
    /// use mhub_event_bus::EventBus;
    ///
    /// #[derive(Clone, Debug, PartialEq)]
    /// struct Tick(u64);
    ///
    /// # fn main() -> Result<(), mhub_event_bus::EventBusError> {
    /// let bus = EventBus::new();
    /// let _rx = bus.subscribe_with_capacity::<Tick>(16)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn subscribe_with_capacity<T: Event>(
        &self,
        capacity: usize,
    ) -> Result<broadcast::Receiver<Arc<T>>, EventBusError> {
        let capacity = validate_capacity(capacity)?;
        let sender = self.ensure_channel::<T>(ChannelKind::Broadcast { capacity }, None)?;
        match sender {
            ChannelHandle::Broadcast(tx) => Ok(tx.subscribe()),
            ChannelHandle::Watch(_) => Err(EventBusError::TypeMismatch {
                message: std::any::type_name::<T>().into(),
                context: Some("Unexpected event type".into()),
            }),
        }
    }

    /// Subscribe to a bounded MPSC channel (queue semantics).
    ///
    /// # Errors
    /// Returns [`EventBusError::ChannelKindMismatch`] if a different channel kind
    /// was already registered for `T`, if the receiver was already taken, or
    /// [`EventBusError::InvalidCapacity`] if `capacity` is zero.
    ///
    /// # Examples
    /// ```rust
    /// use mhub_event_bus::EventBus;
    ///
    /// #[derive(Clone, Debug, PartialEq)]
    /// struct Job(u64);
    ///
    /// # fn main() -> Result<(), mhub_event_bus::EventBusError> {
    /// let bus = EventBus::new();
    /// let _rx = bus.subscribe_mpsc::<Job>(8)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn subscribe_mpsc<T: Event>(
        &self,
        capacity: usize,
    ) -> Result<mpsc::Receiver<Arc<T>>, EventBusError> {
        let capacity = validate_capacity(capacity)?;
        self.take_mpsc_receiver::<T>(capacity)
    }

    /// Subscribe to a watch channel (latest-value semantics). Initializes with the provided value if absent.
    ///
    /// # Errors
    /// Returns [`EventBusError::ChannelKindMismatch`] if a different channel kind
    /// was already registered for `T`.
    ///
    /// # Examples
    /// ```rust
    /// use mhub_event_bus::EventBus;
    ///
    /// #[derive(Clone, Debug, PartialEq)]
    /// struct Snapshot(u64);
    ///
    /// # fn main() -> Result<(), mhub_event_bus::EventBusError> {
    /// let bus = EventBus::new();
    /// let _rx = bus.subscribe_watch::<Snapshot>(Snapshot(0))?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn subscribe_watch<T: Event>(
        &self,
        initial: T,
    ) -> Result<watch::Receiver<Arc<T>>, EventBusError> {
        let sender = self.ensure_channel::<T>(ChannelKind::Watch, Some(Arc::new(initial)))?;
        match sender {
            ChannelHandle::Watch(tx) => Ok(tx.subscribe()),
            ChannelHandle::Broadcast(_) => Err(EventBusError::TypeMismatch {
                message: std::any::type_name::<T>().into(),
                context: Some("Unexpected event type".into()),
            }),
        }
    }

    /// Publishes a shared event instance via broadcast.
    ///
    /// # Errors
    /// Returns [`EventBusError::ChannelKindMismatch`] if a different channel kind
    /// was already registered for `T`.
    ///
    /// # Examples
    /// ```rust
    /// use mhub_event_bus::EventBus;
    ///
    /// #[derive(Clone, Debug, PartialEq)]
    /// struct Ping;
    ///
    /// # fn main() -> Result<(), mhub_event_bus::EventBusError> {
    /// let bus = EventBus::new();
    /// bus.publish(Ping)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn publish<T: Event>(&self, event: T) -> Result<usize, EventBusError> {
        self.publish_arc(Arc::new(event))
    }

    /// Publishes a shared event instance via broadcast without re-wrapping.
    ///
    /// # Errors
    /// Returns [`EventBusError::ChannelKindMismatch`] if a different channel kind
    /// was already registered for `T`.
    ///
    /// # Examples
    /// ```rust
    /// use mhub_event_bus::EventBus;
    /// use std::sync::Arc;
    ///
    /// #[derive(Clone, Debug, PartialEq)]
    /// struct Ping;
    ///
    /// # fn main() -> Result<(), mhub_event_bus::EventBusError> {
    /// let bus = EventBus::new();
    /// bus.publish_arc(Arc::new(Ping))?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn publish_arc<T: Event>(&self, event: Arc<T>) -> Result<usize, EventBusError> {
        let sender =
            self.ensure_channel::<T>(ChannelKind::Broadcast { capacity: DEFAULT_CAPACITY }, None)?;
        let sender = match sender {
            ChannelHandle::Broadcast(tx) => tx,
            ChannelHandle::Watch(_) => {
                return Err(EventBusError::TypeMismatch {
                    message: std::any::type_name::<T>().into(),
                    context: Some("Unexpected event type".into()),
                });
            },
        };

        sender.send(event).map_or_else(
            |_| {
                trace!(event = std::any::type_name::<T>(), "Event dropped: no active subscribers");
                Ok(0)
            },
            |count| {
                trace!(event = std::any::type_name::<T>(), count, "Event dispatched");
                Ok(count)
            },
        )
    }

    /// Publishes to a bounded MPSC channel (queue semantics).
    ///
    /// # Errors
    /// Returns [`EventBusError::ChannelKindMismatch`] if a different channel kind
    /// was already registered for `T`, or [`EventBusError::ChannelFull`] if full.
    ///
    /// # Examples
    /// ```rust
    /// use mhub_event_bus::EventBus;
    ///
    /// #[derive(Clone, Debug, PartialEq)]
    /// struct Job(u64);
    ///
    /// # fn main() -> Result<(), mhub_event_bus::EventBusError> {
    /// let bus = EventBus::new();
    /// let rx = bus.subscribe_mpsc::<Job>(8)?;
    /// bus.publish_mpsc(Job(1))?;
    /// drop(rx);
    /// # Ok(())
    /// # }
    /// ```
    pub fn publish_mpsc<T: Event>(&self, event: T) -> Result<(), EventBusError> {
        self.publish_mpsc_arc(Arc::new(event))
    }

    /// Publishes to a bounded MPSC channel without re-wrapping.
    ///
    /// # Errors
    /// Returns [`EventBusError::ChannelKindMismatch`] if a different channel kind
    /// was already registered for `T`, or [`EventBusError::ChannelFull`] if full.
    ///
    /// # Examples
    /// ```rust
    /// use mhub_event_bus::EventBus;
    /// use std::sync::Arc;
    ///
    /// #[derive(Clone, Debug, PartialEq)]
    /// struct Job(u64);
    ///
    /// # fn main() -> Result<(), mhub_event_bus::EventBusError> {
    /// let bus = EventBus::new();
    /// let rx = bus.subscribe_mpsc::<Job>(8)?;
    /// bus.publish_mpsc_arc(Arc::new(Job(1)))?;
    /// drop(rx);
    /// # Ok(())
    /// # }
    /// ```
    pub fn publish_mpsc_arc<T: Event>(&self, event: Arc<T>) -> Result<(), EventBusError> {
        let sender = self.get_or_create_mpsc::<T>(DEFAULT_CAPACITY)?;
        sender.try_send(event).map_err(|e| EventBusError::ChannelFull {
            message: e.to_string().into(),
            context: Some(std::any::type_name::<T>().into()),
        })
    }

    /// Publishes to a watch channel (latest-value semantics). Creates a channel if missing.
    ///
    /// # Errors
    /// Returns [`EventBusError::ChannelKindMismatch`] if a different channel kind
    /// was already registered for `T`.
    ///
    /// # Examples
    /// ```rust
    /// use mhub_event_bus::EventBus;
    ///
    /// #[derive(Clone, Debug, PartialEq)]
    /// struct Snapshot(u64);
    ///
    /// # fn main() -> Result<(), mhub_event_bus::EventBusError> {
    /// let bus = EventBus::new();
    /// bus.publish_watch(Snapshot(1))?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn publish_watch<T: Event>(&self, event: T) -> Result<(), EventBusError> {
        self.publish_watch_arc(Arc::new(event))
    }

    /// Publishes to a watch channel without re-wrapping. Creates a channel if missing.
    ///
    /// # Errors
    /// Returns [`EventBusError::ChannelKindMismatch`] if a different channel kind
    /// was already registered for `T`.
    ///
    /// # Examples
    /// ```rust
    /// use mhub_event_bus::EventBus;
    /// use std::sync::Arc;
    ///
    /// #[derive(Clone, Debug, PartialEq)]
    /// struct Snapshot(u64);
    ///
    /// # fn main() -> Result<(), mhub_event_bus::EventBusError> {
    /// let bus = EventBus::new();
    /// bus.publish_watch_arc(Arc::new(Snapshot(1)))?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn publish_watch_arc<T: Event>(&self, event: Arc<T>) -> Result<(), EventBusError> {
        let arc = event;
        let sender = self.ensure_channel::<T>(ChannelKind::Watch, Some(arc.clone()))?;
        let sender = match sender {
            ChannelHandle::Watch(tx) => tx,
            ChannelHandle::Broadcast(_) => {
                return Err(EventBusError::TypeMismatch {
                    message: std::any::type_name::<T>().into(),
                    context: Some("Unexpected event type".into()),
                });
            },
        };
        sender.send_replace(arc);
        Ok(())
    }

    /// Gracefully shuts down the bus by dropping all underlying channels.
    ///
    /// Returns the number of event channels that were closed.
    #[must_use]
    pub fn shutdown(&self) -> usize {
        {
            let mut channels = self.channels.write();
            let count = channels.len();
            channels.clear();
            count
        }
    }

    fn ensure_channel<T: Event>(
        &self,
        kind: ChannelKind,
        watch_initial: Option<Arc<T>>,
    ) -> Result<ChannelHandle<T>, EventBusError> {
        let id = TypeId::of::<T>();

        let mut watch_initial = if matches!(kind, ChannelKind::Watch) {
            Some(watch_initial.ok_or_else(|| EventBusError::TypeMismatch {
                message: "Watch channel requires an initial value".into(),
                context: Some(std::any::type_name::<T>().into()),
            })?)
        } else {
            None
        };

        let existing_handle = {
            let channels = self.channels.read();
            channels.get(&id).map(|existing| {
                let result = match (existing.kind, kind) {
                    (
                        ChannelKind::Broadcast { capacity: existing_capacity },
                        ChannelKind::Broadcast { capacity },
                    ) => {
                        if existing_capacity != capacity {
                            warn!(
                                event = std::any::type_name::<T>(),
                                existing_capacity,
                                requested_capacity = capacity,
                                "Broadcast channel already initialized with a different capacity"
                            );
                        }
                        ChannelHandle::from_state(kind, existing)
                    },
                    (
                        ChannelKind::Mpsc { capacity: existing_capacity },
                        ChannelKind::Mpsc { capacity },
                    ) => {
                        if existing_capacity != capacity {
                            warn!(
                                event = std::any::type_name::<T>(),
                                existing_capacity,
                                requested_capacity = capacity,
                                "MPSC channel already initialized with a different capacity"
                            );
                        }
                        ChannelHandle::from_state(kind, existing)
                    },
                    (ChannelKind::Watch, ChannelKind::Watch) => {
                        ChannelHandle::from_state(kind, existing)
                    },
                    _ => Err(EventBusError::ChannelKindMismatch {
                        message: format!(
                            "Expected {:?} but found {:?} for {}",
                            kind,
                            existing.kind,
                            std::any::type_name::<T>()
                        )
                        .into(),
                        context: None,
                    }),
                };
                Some(result)
            })
        };

        if let Some(handle) = existing_handle.flatten() {
            return handle;
        }

        let handle = {
            let mut channels = self.channels.write();
            let handle = {
                let entry = channels.entry(id).or_insert_with(|| {
                    trace!(
                        event = std::any::type_name::<T>(),
                        ?kind,
                        "Initializing new event channel"
                    );
                    let sender: Box<dyn Any + Send + Sync> = match kind {
                        ChannelKind::Broadcast { capacity } => {
                            let (tx, _) = broadcast::channel::<Arc<T>>(capacity);
                            Box::new(tx)
                        },
                        ChannelKind::Mpsc { capacity } => {
                            let (tx, _) = mpsc::channel::<Arc<T>>(capacity);
                            Box::new(tx)
                        },
                        ChannelKind::Watch => {
                            let initial = watch_initial
                                .take()
                                .expect("Watch channel requires an initial value");
                            let (tx, _) = watch::channel::<Arc<T>>(initial);
                            Box::new(tx)
                        },
                    };
                    ChannelState { kind, sender }
                });

                ChannelHandle::from_state(kind, entry)?
            };
            drop(channels);
            handle
        };

        Ok(handle)
    }

    fn get_or_create_mpsc<T: Event>(
        &self,
        capacity: usize,
    ) -> Result<mpsc::Sender<Arc<T>>, EventBusError> {
        let capacity = validate_capacity(capacity)?;
        let id = TypeId::of::<T>();
        let existing = {
            let mut channels = self.channels.write();
            match channels.get_mut(&id) {
                Some(existing) => match existing.kind {
                    ChannelKind::Mpsc { .. } => {
                        match existing.sender.downcast_mut::<MpscChannel<T>>() {
                            Some(chan) => {
                                if let ChannelKind::Mpsc { capacity: existing_capacity } =
                                    existing.kind
                                    && existing_capacity != capacity
                                {
                                    warn!(
                                        event = std::any::type_name::<T>(),
                                        existing_capacity,
                                        requested_capacity = capacity,
                                        "MPSC channel already initialized with a different capacity"
                                    );
                                }
                                Some(Ok(chan.sender.clone()))
                            },
                            None => Some(Err(EventBusError::TypeMismatch {
                                message: std::any::type_name::<T>().into(),
                                context: Some("Unexpected event type".into()),
                            })),
                        }
                    },
                    other => Some(Err(EventBusError::ChannelKindMismatch {
                        message: format!(
                            "Expected Mpsc but found {:?} for {}",
                            other,
                            std::any::type_name::<T>()
                        )
                        .into(),
                        context: None,
                    })),
                },
                None => None,
            }
        };

        if let Some(result) = existing {
            return result;
        }

        let (tx, rx, kind) = {
            trace!(event = std::any::type_name::<T>(), capacity, "Initializing new mpsc channel");
            let (tx, rx) = mpsc::channel::<Arc<T>>(capacity);
            (tx, rx, ChannelKind::Mpsc { capacity })
        };

        {
            let mut channels = self.channels.write();
            let channel = MpscChannel { sender: tx.clone(), receiver: Some(rx), taken: false };
            channels.insert(id, ChannelState { kind, sender: Box::new(channel) });
        }

        Ok(tx)
    }

    fn take_mpsc_receiver<T: Event>(
        &self,
        capacity: usize,
    ) -> Result<mpsc::Receiver<Arc<T>>, EventBusError> {
        let capacity = validate_capacity(capacity)?;
        let id = TypeId::of::<T>();
        let existing = {
            let mut channels = self.channels.write();
            match channels.get_mut(&id) {
                Some(existing) => match existing.kind {
                    ChannelKind::Mpsc { .. } => {
                        match existing.sender.downcast_mut::<MpscChannel<T>>() {
                            Some(chan) => {
                                if let ChannelKind::Mpsc { capacity: existing_capacity } =
                                    existing.kind
                                    && existing_capacity != capacity
                                {
                                    warn!(
                                        event = std::any::type_name::<T>(),
                                        existing_capacity,
                                        requested_capacity = capacity,
                                        "MPSC channel already initialized with a different capacity"
                                    );
                                }
                                if chan.taken {
                                    Some(Err(EventBusError::ChannelKindMismatch {
                                        message: "MPSC receiver already taken".into(),
                                        context: Some(std::any::type_name::<T>().into()),
                                    }))
                                } else {
                                    chan.taken = true;
                                    Some(chan.receiver.take().ok_or_else(|| {
                                        EventBusError::ChannelNotFound {
                                            message: "MPSC receiver missing".into(),
                                            context: Some(std::any::type_name::<T>().into()),
                                        }
                                    }))
                                }
                            },
                            None => Some(Err(EventBusError::TypeMismatch {
                                message: std::any::type_name::<T>().into(),
                                context: Some("Unexpected event type".into()),
                            })),
                        }
                    },
                    other => Some(Err(EventBusError::ChannelKindMismatch {
                        message: format!(
                            "Expected Mpsc but found {:?} for {}",
                            other,
                            std::any::type_name::<T>()
                        )
                        .into(),
                        context: None,
                    })),
                },
                None => None,
            }
        };

        if let Some(result) = existing {
            return result;
        }

        let (tx, rx, kind) = {
            trace!(event = std::any::type_name::<T>(), capacity, "Initializing new mpsc channel");
            let (tx, rx) = mpsc::channel::<Arc<T>>(capacity);
            (tx, rx, ChannelKind::Mpsc { capacity })
        };

        {
            let mut channels = self.channels.write();
            let channel = MpscChannel { sender: tx, receiver: None, taken: true };
            channels.insert(id, ChannelState { kind, sender: Box::new(channel) });
        }

        Ok(rx)
    }
}

fn validate_capacity(capacity: usize) -> Result<usize, EventBusError> {
    if capacity < MIN_CAPACITY {
        return Err(EventBusError::InvalidCapacity {
            message: format!("capacity must be >= {MIN_CAPACITY}").into(),
            context: None,
        });
    }
    Ok(capacity)
}
