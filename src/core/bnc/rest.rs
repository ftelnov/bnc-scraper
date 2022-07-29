use super::config::BncCfg;
use super::error::BncResult;
use super::snapshot::SnapshotFetcher;
use super::snapshot::SymbolSnapshot;
use crate::core::bnc::data::SymbolContainer;
use async_trait::async_trait;
use reqwest::Client;

#[derive(Debug, Clone)]
pub struct BncRestClient {
    base_url: String,
    client: Client,
}

impl BncRestClient {
    pub fn new(client: Client, base_url: String) -> Self {
        Self { client, base_url }
    }

    pub fn from_cfg(cfg: &BncCfg) -> Self {
        Self::new(Client::new(), cfg.baseurl.clone())
    }

    /// Get full path for the given relative path.
    ///
    /// Basically concatenation of base url and given str
    ///
    /// You definitely should provide starting slash and it's better to avoid trailing slash.
    fn rel_path(&self, rel: &str) -> String {
        format!("{}{}", self.base_url, rel)
    }
}

#[async_trait]
impl SnapshotFetcher for BncRestClient {
    async fn fetch_snapshot(&self, symbol: &str) -> BncResult<SymbolSnapshot> {
        let path = self.rel_path("/api/v3/depth");

        let request = self
            .client
            .get(&path)
            .query(&SymbolContainer { symbol })
            .build()?;

        let response = self.client.execute(request).await?.json().await?;
        Ok(response)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::AppCfg;
    use anyhow::Result;

    struct TestCtx {
        client: BncRestClient,
        symbol: String,
    }

    impl TestCtx {
        fn new() -> Self {
            let cfg = AppCfg::load().unwrap();
            Self {
                client: BncRestClient::from_cfg(cfg.core.bnc()),
                symbol: "BTCUSDT".into(),
            }
        }
    }

    // We are satisfied even if this is not panicking behaviour - deserialize and we have a deal here.
    #[tokio::test]
    async fn it_gets_normal_snapshot() -> Result<()> {
        let ctx = TestCtx::new();
        let _ = ctx.client.fetch_snapshot(&ctx.symbol).await?;

        Ok(())
    }

    #[tokio::test]
    async fn it_tries_missing_symbol_snapshot() -> Result<()> {
        let ctx = TestCtx::new();
        let snapshot = ctx.client.fetch_snapshot("NOTFOUND").await;
        assert!(snapshot.is_err());

        Ok(())
    }
}
