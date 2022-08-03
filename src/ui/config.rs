use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct UICfg {
    /// Milliseconds between screen updates
    pub tick_rate: u64,
}

impl Default for UICfg {
    fn default() -> Self {
        Self { tick_rate: 100 }
    }
}
