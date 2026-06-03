use serde::{Deserialize, Serialize};

/// Paginated result set.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Paginator<T> {
    pub data: Vec<T>,
    pub total: u64,
    pub per_page: u64,
    pub current_page: u64,
    pub last_page: u64,
    pub from: u64,
    pub to: u64,
    pub has_more: bool,
}

impl<T> Paginator<T> {
    pub fn new(data: Vec<T>, total: u64, per_page: u64, current_page: u64) -> Self {
        let last_page = ((total as f64) / (per_page as f64)).ceil() as u64;
        let last_page = last_page.max(1);
        let from = (current_page.saturating_sub(1)) * per_page + 1;
        let to = (from + data.len() as u64).saturating_sub(1);
        let has_more = current_page < last_page;

        Self { data, total, per_page, current_page, last_page, from, to, has_more }
    }
}
