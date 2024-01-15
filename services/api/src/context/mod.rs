#[cfg(feature = "plus")]
use bencher_billing::Biller;
#[cfg(feature = "plus")]
use bencher_github::GitHub;
#[cfg(feature = "plus")]
use bencher_license::Licensor;
use bencher_token::TokenKey;
use tokio::sync::mpsc::Sender;
use url::Url;

mod database;
mod messenger;
mod rbac;

pub use database::{DataStoreError, Database, DbConnection};
#[cfg(feature = "plus")]
pub use messenger::ServerStatsBody;
pub use messenger::{Body, ButtonBody, Email, Message, Messenger, NewUserBody};
pub use rbac::{Rbac, RbacError};

#[cfg(feature = "plus")]
use crate::config::plus::StatsSettings;

pub struct ApiContext {
    pub endpoint: Url,
    pub token_key: TokenKey,
    pub rbac: Rbac,
    pub messenger: Messenger,
    pub database: Database,
    pub restart_tx: Sender<()>,
    #[cfg(feature = "plus")]
    pub github: Option<GitHub>,
    #[cfg(feature = "plus")]
    pub stats: StatsSettings,
    #[cfg(feature = "plus")]
    pub biller: Option<Biller>,
    #[cfg(feature = "plus")]
    pub licensor: Licensor,
}

impl ApiContext {
    pub async fn conn(&self) -> tokio::sync::MutexGuard<DbConnection> {
        self.database.connection.lock().await
    }

    #[cfg(feature = "plus")]
    pub fn is_bencher_cloud(&self) -> bool {
        bencher_json::is_bencher_cloud(&self.endpoint) && self.biller.is_some()
    }

    #[cfg(feature = "plus")]
    pub fn biller(&self) -> Result<&Biller, dropshot::HttpError> {
        self.biller.as_ref().ok_or_else(|| {
            crate::error::locked_error("Tried to use a Bencher Cloud route when Self-Hosted")
        })
    }
}
