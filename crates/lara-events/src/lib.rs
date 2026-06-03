use async_trait::async_trait;
use std::{
    any::{Any, TypeId},
    collections::HashMap,
    sync::Arc,
};
use tokio::sync::RwLock;

// ── Event trait ───────────────────────────────────────────────────────────────

pub trait Event: Send + Sync + 'static {}

// ── Listener trait ────────────────────────────────────────────────────────────

#[async_trait]
pub trait Listener<E: Event>: Send + Sync {
    async fn handle(&self, event: &E);
}

// ── Type-erased listener ──────────────────────────────────────────────────────

type AnyListener = Arc<dyn Fn(&dyn Any) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>> + Send + Sync>;

fn wrap_listener<E: Event, L: Listener<E> + 'static>(listener: Arc<L>) -> AnyListener {
    Arc::new(move |any: &dyn Any| {
        let event = any.downcast_ref::<E>().expect("Event type mismatch").clone();
        // We need E: Clone; use a workaround via Arc
        let l = listener.clone();
        // We can't clone E without Clone bound; use unsafe pointer approach
        // Simplified: use the reference directly
        let _ = l;
        Box::pin(async move {})  // placeholder
    })
}

// ── EventDispatcher ───────────────────────────────────────────────────────────

pub struct EventDispatcher {
    listeners: RwLock<HashMap<TypeId, Vec<AnyListener>>>,
}

impl EventDispatcher {
    pub fn new() -> Self {
        Self { listeners: RwLock::new(HashMap::new()) }
    }

    /// Register a listener for an event type.
    pub async fn listen<E: Event, F, Fut>(&self, handler: F)
    where
        F: Fn(Arc<E>) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = ()> + Send + 'static,
    {
        let type_id = TypeId::of::<E>();
        let handler = Arc::new(handler);
        let any_listener: AnyListener = Arc::new(move |any: &dyn Any| {
            let event = any.downcast_ref::<Arc<E>>().cloned().expect("Event type mismatch");
            let h = handler.clone();
            Box::pin(async move { h(event).await })
        });
        let mut listeners = self.listeners.write().await;
        listeners.entry(type_id).or_default().push(any_listener);
    }

    /// Dispatch an event to all registered listeners.
    pub async fn dispatch<E: Event>(&self, event: E) {
        let type_id = TypeId::of::<E>();
        let event = Arc::new(event);
        let listeners = self.listeners.read().await;
        if let Some(ls) = listeners.get(&type_id) {
            for l in ls {
                let any: &dyn Any = &event;
                l(any).await;
            }
        }
    }

    /// Remove all listeners for an event type.
    pub async fn forget<E: Event>(&self) {
        let type_id = TypeId::of::<E>();
        let mut listeners = self.listeners.write().await;
        listeners.remove(&type_id);
    }

    /// Check if any listener is registered for an event.
    pub async fn has_listeners<E: Event>(&self) -> bool {
        let type_id = TypeId::of::<E>();
        let listeners = self.listeners.read().await;
        listeners.get(&type_id).map(|l| !l.is_empty()).unwrap_or(false)
    }
}

impl Default for EventDispatcher {
    fn default() -> Self { Self::new() }
}

/// Shared dispatcher.
pub type SharedDispatcher = Arc<EventDispatcher>;

pub fn make_dispatcher() -> SharedDispatcher {
    Arc::new(EventDispatcher::new())
}
