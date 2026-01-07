//! Slice registry for modular features.
//! This provides a minimal type-erased container for the pre-initialized feature state.

use std::any::{Any, TypeId};
use std::fmt::Debug;

/// Marker trait for feature state that can be shared across threads.
pub trait FeatureSlice: Any + Debug + Send + Sync {
    /// Helper to allow downcasting from the trait object.
    fn as_any(&self) -> &dyn Any;
}

/// A container for an initialized feature.
#[derive(Debug)]
pub struct InitializedSlice {
    pub id: TypeId,
    pub state: Box<dyn FeatureSlice>,
}

impl InitializedSlice {
    /// Create a new initialized slice from a concrete state.
    pub fn new<T: FeatureSlice>(state: T) -> Self {
        Self { id: TypeId::of::<T>(), state: Box::new(state) }
    }
}
