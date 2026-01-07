use axum::extract::FromRef;
use fxhash::FxHashMap;
use mhub_database::Database;
use mhub_domain::config::ApiConfig;
use mhub_domain::registry::{FeatureSlice, InitializedSlice};
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
    pub config: ApiConfig,
    pub database: Database,
    pub events: EventBus,
    //    pub storage: Storage,
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

    #[must_use]
    pub fn get_slice<T: FeatureSlice>(&self) -> Option<&T> {
        self.inner
            .slices
            .get(&TypeId::of::<T>())
            .and_then(|initialized| initialized.state.as_any().downcast_ref::<T>())
    }

    /// Returns a reference to the slice if it is registered.
    ///
    /// # Errors
    /// Returns an error if the slice is not registered.
    pub fn try_get_slice<T: FeatureSlice>(&self) -> Result<&T, ApiStateError> {
        self.get_slice::<T>().ok_or_else(|| ApiStateError::MissingSlice {
            message: std::any::type_name::<T>().into(),
            context: None,
        })
    }

    /// Iterates over registered slice type IDs (for diagnostics).
    pub fn slice_ids(&self) -> impl Iterator<Item = &TypeId> {
        self.inner.slices.keys()
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
        state.inner.config.clone()
    }
}

impl FromRef<ApiState> for Database {
    fn from_ref(state: &ApiState) -> Self {
        state.inner.database.clone()
    }
}

impl FromRef<ApiState> for EventBus {
    fn from_ref(state: &ApiState) -> Self {
        state.inner.events.clone()
    }
}

#[derive(Debug, Default)]
pub struct ApiStateBuilder {
    config: Option<ApiConfig>,
    database: Option<Database>,
    events: Option<EventBus>,
    slices: FxHashMap<TypeId, InitializedSlice>,
}

impl ApiStateBuilder {
    pub fn config(mut self, config: ApiConfig) -> Self {
        self.config = Some(config);
        self
    }

    pub fn db(mut self, database: Database) -> Self {
        self.database = Some(database);
        self
    }

    pub fn events(mut self, events: EventBus) -> Self {
        self.events = Some(events);
        self
    }

    pub fn register_slice(mut self, slice: InitializedSlice) -> Self {
        self.slices.insert(slice.id, slice);
        self
    }

    /// Registers multiple slices at once.
    pub fn register_slices<I>(mut self, slices: I) -> Self
    where
        I: IntoIterator<Item = InitializedSlice>,
    {
        for slice in slices {
            self.slices.insert(slice.id, slice);
        }
        self
    }

    pub fn build(self) -> Result<ApiState, ApiStateError> {
        let config = self.config.ok_or_else(|| ApiStateError::Validation {
            message: "ApiConfig not provided".into(),
            context: None,
        })?;
        let database = self.database.ok_or_else(|| ApiStateError::Validation {
            message: "Database not provided".into(),
            context: None,
        })?;
        let events = self.events.unwrap_or_default();

        Ok(ApiState {
            inner: Arc::new(ApiStateInner { config, database, events, slices: self.slices }),
        })
    }
}
