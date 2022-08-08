use super::super::error::{BncError, BncResult};
use super::super::ws::worker::price::SymbolPriceUpdate;
use super::super::ws::worker::MessageSender;
use std::sync::Arc;
use tokio::sync::watch::Sender;
use tokio::sync::Mutex;

/// Stream of the implementors can be balanced using specific function.
pub trait BalancedEntity {
    /// Get current update id of the entity to balance it across others.
    fn update_id(&self) -> u64;
}

impl BalancedEntity for SymbolPriceUpdate {
    fn update_id(&self) -> u64 {
        self.id
    }
}

/// State to hold balance data. It will be moved into needed clojure to compare its data with new entries.
///
/// MessageSender is implemented for the shared state of message balancer.
#[derive(Debug)]
pub struct MessageBalancer<T> {
    last_update_id: Option<u64>,
    sender: Sender<T>,
}

impl<T> MessageBalancer<T> {
    pub fn new(sender: Sender<T>) -> Self {
        Self {
            last_update_id: None,
            sender,
        }
    }
}

/// We implement sending messages that could be balanced(e.g. implements Balanced trait) for shared MessageBalancer state.
#[async_trait::async_trait]
impl<B: BalancedEntity + Send + Sync> MessageSender<B> for Arc<Mutex<MessageBalancer<B>>> {
    async fn send(&self, data: B) -> BncResult<()> {
        let mut balancer = self.lock().await;
        if let Some(ref mut last_update_id) = balancer.last_update_id {
            if data.update_id() > *last_update_id {
                *last_update_id = data.update_id();
            } else {
                return Err(BncError::DataRejected);
            }
        } else {
            balancer.last_update_id = Some(data.update_id());
        }

        balancer
            .sender
            .send(data)
            .map_err(|_| BncError::DataTransmitError)?;

        Ok(())
    }
}
