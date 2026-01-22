use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Deserialize, PartialEq, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryOrder {
    Asc,
    Desc
}

#[derive(Serialize, ToSchema)]
pub struct PagingResponse<T> {
    items: Vec<T>,
    total: u64,
    has_next: bool,
}

impl<T> PagingResponse<T> {
    pub fn new(items: Vec<T>, total: u64, page: u32, limit: u32) -> Self {
        let has_next = (page as u64 * limit as u64) < total;
        Self {
            items,
            total,
            has_next,
        }
    }
}

#[derive(Serialize, ToSchema)]
pub struct CursorPagingResponse<T> {
    items: Vec<T>,
    next_cursor: Option<String>
}

impl<T> CursorPagingResponse<T> {
    pub fn new(items: Vec<T>, next_cursor: Option<String>) -> Self {
        Self { items, next_cursor }
    }
}