use crate::system::registry::{FeatureSlice, InitializedSlice};
use axum::extract::FromRef;
use fxhash::FxHashMap;
use mhub_database::Database;
use mhub_domain::config::ApiConfig;
use mhub_event_bus::EventBus;
use std::any::TypeId;
use std::borrow::Cow;
use std::ops::Deref;
use std::sync::Arc;

#[mhub_derive::mhub_error]
pub enum ApiStateError {
    #[error("State validation error{}: {message}", format_context(.context))]
    Validation { message: Cow<'static, str>, context: Option<Cow<'static, str>> },
    #[error("State missing feature slice{}: {message}", format_context(.context))]
    MissingSlice { message: Cow<'static, str>, context: Option<Cow<'static, str>> },
}

#[derive(Debug)]
pub struct ApiStateInner {
    pub cfg: ApiConfig,
    pub db: Database,
    pub events: EventBus,
    slices: FxHashMap<TypeId, InitializedSlice>,
}

#[derive(Debug, Clone)]
pub struct ApiState {
    inner: Arc<ApiStateInner>,
}

impl ApiState {
    #[must_use]
    pub fn builder() -> ApiStateBuilder {
        ApiStateBuilder::default()
    }

    pub fn get_slice<T: FeatureSlice>(&self) -> Option<&T> {
        self.inner
            .slices
            .get(&TypeId::of::<T>())
            .and_then(|initialized| initialized.state.as_any().downcast_ref::<T>())
    }

    pub fn try_get_slice<T: FeatureSlice>(&self) -> Result<&T> {
        self.get_slice::<T>().ok_or_else(|| ApiStateError::MissingSlice {
            message: std::any::type_name::<T>().into(),
            context: None,
        })
    }
}

impl Deref for ApiState {
    type Target = ApiStateInner;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl FromRef<ApiState> for ApiConfig {
    fn from_ref(state: &ApiState) -> Self {
        state.inner.cfg.clone()
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

#[derive(Debug, Default)]
pub struct ApiStateBuilder {
    cfg: Option<ApiConfig>,
    db: Option<Database>,
    events: Option<EventBus>,
    slices: FxHashMap<TypeId, InitializedSlice>,
}

impl ApiStateBuilder {
    pub fn with_config(mut self, cfg: ApiConfig) -> Self {
        self.cfg = Some(cfg);
        self
    }

    pub fn with_db(mut self, db: Database) -> Self {
        self.db = Some(db);
        self
    }

    pub fn with_events(mut self, events: EventBus) -> Self {
        self.events = Some(events);
        self
    }

    pub fn register_slice(mut self, slice: InitializedSlice) -> Self {
        self.slices.insert(slice.id, slice);
        self
    }

    pub fn build(self) -> Result<ApiState> {
        let cfg = self.cfg.ok_or_else(|| ApiStateError::Validation {
            message: "ApiConfig not provided".into(),
            context: None,
        })?;
        let db = self.db.ok_or_else(|| ApiStateError::Validation {
            message: "Database not provided".into(),
            context: None,
        })?;
        let events = self.events.unwrap_or_default();

        Ok(ApiState { inner: Arc::new(ApiStateInner { cfg, db, events, slices: self.slices }) })
    }
}
