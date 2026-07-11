//! Core application abstractions and dependency injection.

pub mod app;
pub mod traits;

pub use app::AppContext;
pub use traits::{EventBus, InMemoryEventBus, Service};
