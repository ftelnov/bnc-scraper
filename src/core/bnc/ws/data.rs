use serde::Deserialize;

/// Data container simply holds some serde value.
#[derive(Debug, Deserialize, Clone)]
pub struct WsDataContainer<T> {
    pub data: T,
}
