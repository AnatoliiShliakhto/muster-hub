use std::any::{Any, TypeId};
use std::fmt::Debug;

/// Marker trait for feature state that can be shared across threads.
pub trait FeatureSlice: Any + Debug + Send + Sync {
    /// Helper to allow downcasting from the trait object.
    fn as_any(&self) -> &dyn Any;
}

/// A container for an initialized feature.
pub struct InitializedSlice {
    pub id: TypeId,
    pub state: Box<dyn FeatureSlice>,
}

impl InitializedSlice {
    pub fn new<T: FeatureSlice>(state: T) -> Self {
        Self { id: TypeId::of::<T>(), state: Box::new(state) }
    }
}

impl Debug for InitializedSlice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InitializedSlice")
            .field("id", &self.id)
            .field("state", &self.state)
            .finish()
    }
}
