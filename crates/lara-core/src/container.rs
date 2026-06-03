use std::{
    any::{Any, TypeId},
    collections::HashMap,
    sync::Arc,
};
use tokio::sync::RwLock;
use crate::error::{CoreError, Result};

type AnyFactory = Box<dyn Fn(&Container) -> Box<dyn Any + Send + Sync> + Send + Sync>;

/// IoC Container — binds types/strings to factories or singleton instances.
pub struct Container {
    bindings:  HashMap<String, Arc<AnyFactory>>,
    singletons: HashMap<String, Arc<dyn Any + Send + Sync>>,
    aliases:   HashMap<String, String>,
}

impl Container {
    pub fn new() -> Self {
        Self {
            bindings:   HashMap::new(),
            singletons: HashMap::new(),
            aliases:    HashMap::new(),
        }
    }

    // ── Registration ────────────────────────────────────────────────────────

    /// Bind a factory closure under a string key.
    pub fn bind<F, T>(&mut self, key: &str, factory: F)
    where
        F: Fn(&Container) -> T + Send + Sync + 'static,
        T: Send + Sync + 'static,
    {
        let wrapped: AnyFactory = Box::new(move |c| Box::new(factory(c)));
        self.bindings.insert(key.to_string(), Arc::new(wrapped));
    }

    /// Bind a type by its TypeId.
    pub fn bind_type<T, F>(&mut self, factory: F)
    where
        F: Fn(&Container) -> T + Send + Sync + 'static,
        T: Send + Sync + 'static,
    {
        let key = format!("{:?}", TypeId::of::<T>());
        self.bind(&key, factory);
    }

    /// Bind as a singleton — the factory is called once; subsequent makes return the same instance.
    pub fn singleton<F, T>(&mut self, key: &str, factory: F)
    where
        F: Fn(&Container) -> T + Send + Sync + 'static,
        T: Send + Sync + 'static,
    {
        self.bind(key, factory);
        // mark as singleton by aliasing to itself with a `__singleton:` prefix
        let singleton_key = format!("__singleton:{}", key);
        self.aliases.insert(singleton_key, key.to_string());
    }

    /// Register a pre-built instance as a singleton.
    pub fn instance<T: Send + Sync + 'static>(&mut self, key: &str, value: T) {
        self.singletons.insert(key.to_string(), Arc::new(value));
    }

    /// Create an alias from `alias` → `abstract_key`.
    pub fn alias(&mut self, alias: &str, abstract_key: &str) {
        self.aliases.insert(alias.to_string(), abstract_key.to_string());
    }

    // ── Resolution ───────────────────────────────────────────────────────────

    /// Resolve a binding by string key and downcast to `T`.
    pub fn make<T: Send + Sync + 'static>(&self, key: &str) -> Result<Arc<T>> {
        let resolved_key = self.resolve_alias(key);

        // Check pre-built instances first
        if let Some(instance) = self.singletons.get(resolved_key) {
            return instance.clone().downcast::<T>().map(Ok).unwrap_or_else(|_| {
                Err(CoreError::ResolutionFailed(
                    key.to_string(),
                    "downcast failed".to_string(),
                ))
            });
        }

        // Use factory
        if let Some(factory) = self.bindings.get(resolved_key) {
            let boxed = factory(self);
            let arc: Arc<dyn Any + Send + Sync> = Arc::from(boxed);
            return arc.downcast::<T>().map_err(|_| {
                CoreError::ResolutionFailed(key.to_string(), "downcast failed".to_string())
            });
        }

        Err(CoreError::BindingNotFound(key.to_string()))
    }

    /// Resolve by TypeId.
    pub fn make_type<T: Send + Sync + 'static>(&self) -> Result<Arc<T>> {
        let key = format!("{:?}", TypeId::of::<T>());
        self.make::<T>(&key)
    }

    /// Check whether a key is bound.
    pub fn bound(&self, key: &str) -> bool {
        let resolved = self.resolve_alias(key);
        self.bindings.contains_key(resolved) || self.singletons.contains_key(resolved)
    }

    fn resolve_alias<'a>(&'a self, key: &'a str) -> &'a str {
        self.aliases.get(key).map(|s| s.as_str()).unwrap_or(key)
    }
}

impl Default for Container {
    fn default() -> Self {
        Self::new()
    }
}

/// Thread-safe shared container.
pub type SharedContainer = Arc<RwLock<Container>>;

/// Create a new shared container.
pub fn make_container() -> SharedContainer {
    Arc::new(RwLock::new(Container::new()))
}
