use axum::{
    Router,
    routing::{delete, get, patch, post, put},
    handler::Handler,
};
use tower_http::cors::{Any, CorsLayer};

/// Fluent router builder — wraps Axum's Router with a Laravel-style API.
pub struct LaraRouter {
    inner: Router,
    prefix: String,
}

impl LaraRouter {
    pub fn new() -> Self {
        Self { inner: Router::new(), prefix: String::new() }
    }

    pub fn prefix(mut self, p: &str) -> Self {
        self.prefix = p.to_string();
        self
    }

    fn full_path(&self, path: &str) -> String {
        format!("{}{}", self.prefix, path)
    }

    pub fn get<H, T>(mut self, path: &str, handler: H) -> Self
    where
        H: Handler<T, ()>,
        T: 'static,
    {
        let full = self.full_path(path);
        self.inner = self.inner.route(&full, get(handler));
        self
    }

    pub fn post<H, T>(mut self, path: &str, handler: H) -> Self
    where
        H: Handler<T, ()>,
        T: 'static,
    {
        let full = self.full_path(path);
        self.inner = self.inner.route(&full, post(handler));
        self
    }

    pub fn put<H, T>(mut self, path: &str, handler: H) -> Self
    where
        H: Handler<T, ()>,
        T: 'static,
    {
        let full = self.full_path(path);
        self.inner = self.inner.route(&full, put(handler));
        self
    }

    pub fn patch<H, T>(mut self, path: &str, handler: H) -> Self
    where
        H: Handler<T, ()>,
        T: 'static,
    {
        let full = self.full_path(path);
        self.inner = self.inner.route(&full, patch(handler));
        self
    }

    pub fn delete<H, T>(mut self, path: &str, handler: H) -> Self
    where
        H: Handler<T, ()>,
        T: 'static,
    {
        let full = self.full_path(path);
        self.inner = self.inner.route(&full, delete(handler));
        self
    }

    /// Group routes under a shared prefix / middleware.
    pub fn group(mut self, options: RouteGroup, f: impl FnOnce(LaraRouter) -> LaraRouter) -> Self {
        let sub = LaraRouter::new().prefix(&format!("{}{}", self.prefix, options.prefix));
        let sub = f(sub);
        self.inner = self.inner.merge(sub.inner);
        self
    }

    /// Merge another LaraRouter.
    pub fn merge(mut self, other: LaraRouter) -> Self {
        self.inner = self.inner.merge(other.inner);
        self
    }

    /// Add a CORS layer permitting all origins (development convenience).
    pub fn with_cors(mut self) -> Self {
        self.inner = self.inner.layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        );
        self
    }

    /// Build the inner Axum Router.
    pub fn build(self) -> Router {
        self.inner
    }
}

impl Default for LaraRouter {
    fn default() -> Self { Self::new() }
}

/// Options for a route group.
pub struct RouteGroup {
    pub prefix: String,
    pub middleware: Vec<String>,
}

impl RouteGroup {
    pub fn new() -> Self {
        Self { prefix: String::new(), middleware: Vec::new() }
    }

    pub fn prefix(mut self, p: &str) -> Self {
        self.prefix = p.to_string(); self
    }

    pub fn middleware(mut self, name: &str) -> Self {
        self.middleware.push(name.to_string()); self
    }
}

impl Default for RouteGroup {
    fn default() -> Self { Self::new() }
}
