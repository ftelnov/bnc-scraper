use crate::core::bnc::error::{BncError, BncResult};
use crate::core::bnc::ws::config::WsCfg;
use futures::Stream;
use futures_util::StreamExt;
use tokio::sync::mpsc::Sender as TokioSender;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message;

/// Implementors are to be used in transmitting messages from workers to messages' consumers.
///
/// Just an abstraction to keep things easier.
#[async_trait::async_trait]
pub trait MessageSender<T: Send + Sync>: Send {
    /// Send message to some receiver. Returns error if message is rejected.
    ///
    /// Returns Ok() if message was sent to receiver.
    ///
    /// Implementors must not use blocking utilities inside implementation.
    async fn send(&self, data: T) -> BncResult<()>;
}

#[async_trait::async_trait]
impl<T: Send + Sync> MessageSender<T> for TokioSender<T> {
    async fn send(&self, data: T) -> BncResult<()> {
        self.send(data)
            .await
            .map_err(|_| BncError::DataTransmitError)?;
        Ok(())
    }
}

/// Order book keeping.
pub mod depth;

/// Realtime symbol's best price updating.
pub mod price;

/// WS worker handles realtime updates of the symbol's price.
///
/// It's purpose to schedule listening threads that will send the data to the provided sender.
///
/// It doesn't, however, provide load balancing across child processes - so worker's results may be repeated.
pub struct WsWorker<'a> {
    base_url: &'a str,
}

impl<'a> WsWorker<'a> {
    pub fn new(base_url: &'a str) -> Self {
        Self { base_url }
    }

    pub fn from_cfg(cfg: &'a WsCfg) -> Self {
        Self {
            base_url: &cfg.baseurl,
        }
    }
}

/// Connect to the given stream endpoint, cut undesired messages(like ping, etc) and unwrap errors
async fn bnc_stream_connect(endpoint: &str) -> BncResult<impl Stream<Item = Message>> {
    let (ws_stream, _) = connect_async(endpoint).await?;
    Ok(ws_stream.filter_map(|message| async {
        let message = message.ok()?;
        if message.is_text() {
            Some(message)
        } else {
            None
        }
    }))
}
