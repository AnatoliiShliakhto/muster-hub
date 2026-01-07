use axum::extract::FromRef;
use mhub_db::Database;
use mhub_event_bus::EventBus;
use std::sync::Arc;

pub struct ApiStateInner {
    pub db: Database,
    pub events: EventBus,
}

#[derive(Clone)]
pub struct ApiState {
    inner: Arc<ApiStateInner>,
}

impl ApiState {
    pub fn new(db: Database, events: EventBus) -> Self {
        Self { inner: Arc::new(ApiStateInner { db, events }) }
    }
}

impl FromRef<ApiState> for Database {
    fn from_ref(state: &ApiState) -> Self {
        state.inner.db.clone()
    }
}

impl FromRef<ApiState> for EventBus {
    fn from_ref(state: &ApiState) -> Self {
        state.inner.events.clone()
    }
}
