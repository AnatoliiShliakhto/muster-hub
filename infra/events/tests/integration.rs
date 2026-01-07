pub mod fixtures;

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

        let received = rx.recv().await.unwrap();
        assert_eq!(*received, event);
    }

    #[tokio::test]
    async fn test_receiver_lagged_recovery() {
        let bus = EventBus::new();
        let capacity = 2;
        let mut rx = bus.subscribe_with_capacity::<TestEvent>(capacity).unwrap();

        let total_messages = 100;
        for i in 0..total_messages {
            bus.publish(TestEvent(i)).unwrap();
        }

        let first_received = loop {
            match rx.recv().await {
                Ok(event) => break event,
                Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => {},
                Err(err) => panic!("Should recover from lag: {err:?}"),
            }
        };

        assert!(
            first_received.0 >= (total_messages - capacity),
            "Should have skipped to the fresh tail of the buffer. Expected >= {}, got {}",
            total_messages - capacity,
            first_received.0
        );

        let second_received = rx.recv().await.expect("Should continue receiving");
        assert_eq!(second_received.0, first_received.0 + 1);
    }

    #[tokio::test]
    async fn test_multiple_subscribers_isolation() {
        let bus = EventBus::new();
        let mut rx1 = bus.subscribe::<TestEvent>().unwrap();
        let mut rx2 = bus.subscribe::<TestEvent>().unwrap();

        bus.publish(TestEvent(100)).unwrap();

        let res1 = rx1.recv().await.unwrap();
        let res2 = rx2.recv().await.unwrap();

        assert_eq!(res1.0, res2.0);
    }

    #[tokio::test]
    async fn test_multiple_event_types_are_isolated() {
        #[derive(Clone, Debug, PartialEq, Eq)]
        struct OtherEvent(pub usize);

        let bus = EventBus::new();
        let mut rx_test = bus.subscribe::<TestEvent>().unwrap();
        let mut rx_other = bus.subscribe::<OtherEvent>().unwrap();

        bus.publish(TestEvent(7)).unwrap();
        bus.publish(OtherEvent(13)).unwrap();

        let got_test = rx_test.recv().await.unwrap();
        let got_other = rx_other.recv().await.unwrap();

        assert_eq!(got_test.0, 7);
        assert_eq!(got_other.0, 13);
    }

    #[tokio::test]
    async fn test_bus_closure_detection() {
        let bus = EventBus::new();
        let mut rx = bus.subscribe::<TestEvent>().unwrap();

        drop(bus);

        let result = rx.recv().await;
        assert!(
            matches!(result, Err(tokio::sync::broadcast::error::RecvError::Closed)),
            "receiver should observe bus closure"
        );
    }

    #[tokio::test]
    async fn test_shutdown_closes_all_channels() {
        let bus = EventBus::new();
        let mut rx = bus.subscribe::<TestEvent>().unwrap();

        let closed = bus.shutdown();
        assert_eq!(closed, 1, "expected a single event channel to be closed");

        let result = rx.recv().await;
        assert!(
            matches!(result, Err(tokio::sync::broadcast::error::RecvError::Closed)),
            "receiver should observe channel closure after shutdown"
        );
    }

    #[tokio::test]
    async fn test_mpsc_queue_semantics() {
        let bus = EventBus::new();
        let mut rx = bus.subscribe_mpsc::<TestEvent>(4).unwrap();

        for i in 0..3 {
            bus.publish_mpsc(TestEvent(i)).unwrap();
        }

        let a = rx.recv().await.unwrap();
        let b = rx.recv().await.unwrap();
        let c = rx.recv().await.unwrap();
        assert_eq!(a.0, 0);
        assert_eq!(b.0, 1);
        assert_eq!(c.0, 2);
    }

    #[tokio::test]
    async fn test_publish_arc_variants() {
        use std::sync::Arc;

        #[derive(Clone, Debug, PartialEq, Eq)]
        struct MpscEvent(pub i64);

        #[derive(Clone, Debug, PartialEq, Eq)]
        struct WatchEvent(pub i64);

        let bus = EventBus::new();

        let mut rx = bus.subscribe::<TestEvent>().unwrap();
        let event = Arc::new(TestEvent(10));
        bus.publish_arc(event.clone()).unwrap();
        let received = rx.recv().await.unwrap();
        assert_eq!(received.0, 10);

        let mut mpsc_rx = bus.subscribe_mpsc::<MpscEvent>(2).unwrap();
        let event = Arc::new(MpscEvent(11));
        bus.publish_mpsc_arc(event.clone()).unwrap();
        assert_eq!(mpsc_rx.recv().await.unwrap().0, 11);

        let watch_rx = bus.subscribe_watch::<WatchEvent>(WatchEvent(0)).unwrap();
        let event = Arc::new(WatchEvent(12));
        bus.publish_watch_arc(event).unwrap();
        assert_eq!(watch_rx.borrow().0, 12);
    }

    #[tokio::test]
    async fn test_mpsc_receiver_only_once() {
        let bus = EventBus::new();
        let _rx = bus.subscribe_mpsc::<TestEvent>(1).unwrap();
        let second = bus.subscribe_mpsc::<TestEvent>(1);
        assert!(second.is_err(), "second mpsc receiver should error");
    }

    #[tokio::test]
    async fn test_watch_latest_value() {
        let bus = EventBus::new();
        let rx = bus.subscribe_watch::<TestEvent>(TestEvent(1)).unwrap();

        bus.publish_watch(TestEvent(2)).unwrap();
        bus.publish_watch(TestEvent(3)).unwrap();

        assert_eq!(rx.borrow().0, 3);
    }

    #[tokio::test]
    async fn test_receiver_ext_for_mpsc_and_watch() {
        #[derive(Clone, Debug, PartialEq, Eq)]
        struct MpscEvent(pub i64);

        #[derive(Clone, Debug, PartialEq, Eq)]
        struct WatchEvent(pub i64);

        let bus = EventBus::new();

        let mut mpsc_rx = bus.subscribe_mpsc::<MpscEvent>(4).unwrap();
        bus.publish_mpsc(MpscEvent(9)).unwrap();
        let received = mpsc_rx.recv().await.unwrap();
        assert_eq!(received.0, 9);

        let mut watch_rx = bus.subscribe_watch::<WatchEvent>(WatchEvent(0)).unwrap();
        bus.publish_watch(WatchEvent(10)).unwrap();
        let received = watch_rx.recv().await.unwrap();
        assert_eq!(received.0, 10);
    }

    #[tokio::test]
    async fn test_ordering_is_preserved() {
        let bus = EventBus::new();
        let mut rx = bus.subscribe::<TestEvent>().unwrap();

        for i in 0..100 {
            bus.publish(TestEvent(i)).unwrap();
        }

        for i in 0..100 {
            let event = rx.recv().await.unwrap();
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
        while tokio::time::timeout(std::time::Duration::from_millis(100), rx.recv()).await.is_ok() {
            received += 1;
        }

        assert_eq!(received, 100, "Should receive all events");
    }

    #[tokio::test]
    async fn test_invalid_capacity_rejected() {
        let bus = EventBus::new();

        let result = bus.subscribe_with_capacity::<TestEvent>(0);
        assert!(matches!(result, Err(EventBusError::InvalidCapacity { .. })));

        let result = bus.subscribe_mpsc::<TestEvent>(0);
        assert!(matches!(result, Err(EventBusError::InvalidCapacity { .. })));
    }
}
