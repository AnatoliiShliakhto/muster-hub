use crate::bus::Event;
use std::future::Future;
use std::sync::Arc;
use tokio::sync::{broadcast, mpsc, watch};
use tracing::{debug, warn};

/// An extension trait for event receivers to provide a more ergonomic API.
///
/// For `watch::Receiver`, `recv` waits for a change before returning the
/// latest value.
pub trait EventReceiverExt<T> {
    /// Receive the next event, returning `None` when the channel is closed.
    fn recv(&mut self) -> impl Future<Output = Option<Arc<T>>> + Send;

    /// Receive the next event, returning `None` when the channel is closed.
    ///
    /// This is a convenience alias for [`EventReceiverExt::recv`].
    fn recv_event(&mut self) -> impl Future<Output = Option<Arc<T>>> + Send {
        self.recv()
    }
}

impl<T: Event> EventReceiverExt<T> for broadcast::Receiver<Arc<T>> {
    async fn recv(&mut self) -> Option<Arc<T>> {
        let mut skipped = 0u64;

        loop {
            match self.recv().await {
                Ok(event) => {
                    if skipped > 0 {
                        warn!(
                            event = std::any::type_name::<T>(),
                            skipped = skipped,
                            "EventBus receiver lagged; continuing from latest message"
                        );
                    }
                    return Some(event);
                },
                Err(broadcast::error::RecvError::Lagged(n)) => {
                    skipped = skipped.saturating_add(n);
                    debug!(
                        event = std::any::type_name::<T>(),
                        skipped = n,
                        total_skipped = skipped,
                        "EventBus receiver lagged; accumulating skipped events"
                    );
                },
                Err(broadcast::error::RecvError::Closed) => return None,
            }
        }
    }
}

impl<T: Event> EventReceiverExt<T> for mpsc::Receiver<Arc<T>> {
    async fn recv(&mut self) -> Option<Arc<T>> {
        self.recv().await
    }
}

impl<T: Event> EventReceiverExt<T> for watch::Receiver<Arc<T>> {
    async fn recv(&mut self) -> Option<Arc<T>> {
        match self.changed().await {
            Ok(()) => Some(self.borrow().clone()),
            Err(_) => None,
        }
    }
}
