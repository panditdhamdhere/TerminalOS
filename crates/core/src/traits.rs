use terminalos_shared::Result;

/// Marker trait for injectable services.
pub trait Service: Send + Sync {}

/// Event bus for decoupled communication between components.
pub trait EventBus: Send + Sync {
    fn publish(&self, event: &str, payload: &str) -> Result<()>;
}

/// Simple in-memory event bus for single-process mode.
pub struct InMemoryEventBus;

impl EventBus for InMemoryEventBus {
    fn publish(&self, event: &str, payload: &str) -> terminalos_shared::Result<()> {
        tracing::debug!(event, payload, "event published");
        Ok(())
    }
}
