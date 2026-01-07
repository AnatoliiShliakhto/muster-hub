mod fixtures;

#[cfg(test)]
mod tests {
    use super::fixtures::*;
    use mhub_event_bus::*;

    #[tokio::test]
    async fn test_event_flow() {
        let bus = EventBus::new();
        let mut rx = bus.subscribe::<TestEvent>().unwrap();

        let event = TestEvent(42);
        bus.publish(event.clone()).unwrap();

        let received = rx.recv_event().await.unwrap();
        assert_eq!(*received, event);
    }

    #[tokio::test]
    async fn test_receiver_lagged_recovery() {
        let bus = EventBus::new();
        let capacity = 2;
        let mut rx =
            bus.subscribe_with_capacity::<TestEvent>(capacity).unwrap();

        let total_messages = 100;
        for i in 0..total_messages {
            bus.publish(TestEvent(i)).unwrap();
        }

        let first_received =
            rx.recv_event().await.expect("Should recover from lag");

        assert!(
            first_received.0 >= (total_messages - capacity),
            "Should have skipped to the fresh tail of the buffer. Expected >= {}, got {}",
            total_messages - capacity,
            first_received.0
        );

        let second_received =
            rx.recv_event().await.expect("Should continue receiving");
        assert_eq!(second_received.0, first_received.0 + 1);
    }

    #[tokio::test]
    async fn test_multiple_subscribers_isolation() {
        let bus = EventBus::new();
        let mut rx1 = bus.subscribe::<TestEvent>().unwrap();
        let mut rx2 = bus.subscribe::<TestEvent>().unwrap();

        bus.publish(TestEvent(100)).unwrap();

        let res1 = rx1.recv_event().await.unwrap();
        let res2 = rx2.recv_event().await.unwrap();

        assert_eq!(res1.0, res2.0);
    }

    #[tokio::test]
    async fn test_bus_closure_detection() {
        let bus = EventBus::new();
        let mut rx = bus.subscribe::<TestEvent>().unwrap();

        drop(bus);

        let result = rx.recv_event().await;
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_ordering_is_preserved() {
        let bus = EventBus::new();
        let mut rx = bus.subscribe::<TestEvent>().unwrap();

        for i in 0..100 {
            bus.publish(TestEvent(i)).unwrap();
        }

        for i in 0..100 {
            let event = rx.recv_event().await.unwrap();
            assert_eq!(event.0, i, "Events should arrive in order");
        }
    }

    #[tokio::test]
    async fn test_concurrent_publishers() {
        use std::sync::Arc;

        let bus = Arc::new(EventBus::new());
        let mut rx = bus.subscribe::<TestEvent>().unwrap();

        let bus1 = bus.clone();
        let handle1 = tokio::spawn(async move {
            for i in 0..50 {
                bus1.publish(TestEvent(i)).unwrap();
            }
        });

        let bus2 = bus.clone();
        let handle2 = tokio::spawn(async move {
            for i in 50..100 {
                bus2.publish(TestEvent(i)).unwrap();
            }
        });

        handle1.await.unwrap();
        handle2.await.unwrap();

        let mut received = 0;
        while let Ok(Some(_)) = tokio::time::timeout(
            std::time::Duration::from_millis(100),
            rx.recv_event()
        ).await {
            received += 1;
        }

        assert_eq!(received, 100, "Should receive all events");
    }
}
