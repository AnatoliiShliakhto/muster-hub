# mhub-event-bus 🚌

Async, type-safe event bus built on `tokio` with per-type channels and ergonomic receiver extensions
for broadcast, mpsc, and watch semantics.

## Features

- Type-driven channels keyed by `TypeId`; cloneable `EventBus`.
- Broadcast fan-out, bounded MPSC queues, and watch latest-value channels.
- Zero-copy publish APIs: `publish_*` plus `publish_*_arc`.
- Lag-aware broadcast receiver extension that resumes after dropped messages.
- Optional tracing for dropped/no-subscriber events and capacity mismatches.

## Quick start (broadcast)

```rust
use mhub_event_bus::{EventBus, EventReceiverExt, EventBusError};

#[derive(Clone, Debug, PartialEq)]
struct UserCreated {
    id: u64
}

#[tokio::main]
async fn main() -> Result<(), EventBusError> {
    let bus = EventBus::new();

    let mut rx = bus.subscribe::<UserCreated>()?;
    bus.publish(UserCreated { id: 42 })?;

    if let Some(event) = rx.recv().await {
        assert_eq!(event.id, 42);
    }
    Ok(())
}
```

## MPSC (queue)

```rust
use mhub_event_bus::{EventBus, EventReceiverExt, EventBusError};

#[derive(Clone, Debug, PartialEq)]
struct Job(pub u64);

#[tokio::main]
async fn main() -> Result<(), EventBusError> {
    let bus = EventBus::new();

    let mut rx = bus.subscribe_mpsc::<Job>(16)?;
    bus.publish_mpsc(Job(7))?;

    let job = rx.recv().await.unwrap();
    assert_eq!(job.0, 7);
    Ok(())
}
```

## Watch (latest value)

```rust
use mhub_event_bus::{EventBus, EventReceiverExt, EventBusError};

#[derive(Clone, Debug, PartialEq)]
struct Snapshot(pub u64);

#[tokio::main]
async fn main() -> Result<(), EventBusError> {
    let bus = EventBus::new();

    let mut rx = bus.subscribe_watch::<Snapshot>(Snapshot(0))?;
    bus.publish_watch(Snapshot(3))?;

    let snapshot = rx.recv().await.unwrap();
    assert_eq!(snapshot.0, 3);
    Ok(())
}
```

## Lag handling (broadcast)

- `EventReceiverExt::recv()` (and the `recv_event()` alias) handle `Lagged` by skipping forward to
  the freshest entry.
- A `debug` log is emitted per lag and a `warn` when the receiver catches up.

## Shutdown

- Call `EventBus::shutdown()` to drop all channels; broadcast receivers will observe closure.

## Notes

- A single event type can only be associated with one channel kind; mismatches return
  `EventBusError::ChannelKindMismatch`.
- MPSC receivers are single-consumer; the first `subscribe_mpsc` call wins for a given event type.
- Channel capacities must be greater than zero.

## Testing

- Integration tests cover round-trips, lag recovery, multi-subscriber isolation, shutdown, and
  multi-type isolation.
