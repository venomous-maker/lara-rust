use serde_json::Value as JsonValue;

/// Life-cycle events fired by the ORM around model mutations.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ModelEvent {
    Creating,
    Created,
    Updating,
    Updated,
    Saving,
    Saved,
    Deleting,
    Deleted,
    Restoring,
    Restored,
    Retrieved,
}

/// A hook function for a model event.
pub type EventHook = Box<dyn Fn(&JsonValue) -> bool + Send + Sync>;
// Returns `true` to continue, `false` to abort the operation.

/// Per-model observer that can intercept any event.
pub trait ModelObserver: Send + Sync {
    fn creating(&self, _data: &JsonValue) -> bool { true }
    fn created(&self, _data: &JsonValue) {}
    fn updating(&self, _data: &JsonValue) -> bool { true }
    fn updated(&self, _data: &JsonValue) {}
    fn saving(&self, _data: &JsonValue) -> bool { true }
    fn saved(&self, _data: &JsonValue) {}
    fn deleting(&self, _data: &JsonValue) -> bool { true }
    fn deleted(&self, _data: &JsonValue) {}
    fn restoring(&self, _data: &JsonValue) -> bool { true }
    fn restored(&self, _data: &JsonValue) {}
    fn retrieved(&self, _data: &JsonValue) {}
}
