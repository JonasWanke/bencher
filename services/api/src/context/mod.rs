#[cfg(feature = "plus")]
use bencher_billing::Biller;
#[cfg(feature = "plus")]
use bencher_github::GitHub;
#[cfg(feature = "plus")]
use bencher_license::Licensor;
use bencher_token::TokenKey;
use tokio::sync::mpsc::Sender;
use url::Url;

#[cfg(feature = "plus")]
use crate::config::plus::StatsSettings;
#[cfg(feature = "plus")]
use crate::model::project::QueryProject;

mod database;
#[cfg(feature = "plus")]
mod indexer;
mod messenger;
mod rbac;

pub use database::{DataStoreError, Database, DbConnection};
#[cfg(feature = "plus")]
pub use indexer::Indexer;
#[cfg(feature = "plus")]
pub use messenger::ServerStatsBody;
pub use messenger::{Body, ButtonBody, Email, Message, Messenger, NewUserBody};
pub use rbac::{Rbac, RbacError};

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
    pub indexer: Option<Indexer>,
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

    #[cfg(feature = "plus")]
    pub async fn update_index(&self, log: &slog::Logger, query_project: &QueryProject) {
        let Some(indexer) = &self.indexer else {
            return;
        };

        let url = match query_project.perf_url(&self.endpoint) {
            Ok(Some(url)) => url,
            Ok(None) => return,
            Err(e) => {
                slog::error!(log, "{e}");
                #[cfg(feature = "sentry")]
                sentry::capture_error(&e);
                return;
            },
        };
        if let Err(e) = indexer.updated(url).await {
            slog::error!(log, "{e}");
            #[cfg(feature = "sentry")]
            sentry::capture_error(&e);
        }
    }

    #[cfg(feature = "plus")]
    pub async fn delete_index(&self, log: &slog::Logger, query_project: &QueryProject) {
        let Some(indexer) = &self.indexer else {
            return;
        };

        let url = match query_project.perf_url(&self.endpoint) {
            Ok(Some(url)) => url,
            Ok(None) => return,
            Err(e) => {
                slog::error!(log, "{e}");
                #[cfg(feature = "sentry")]
                sentry::capture_error(&e);
                return;
            },
        };
        if let Err(e) = indexer.deleted(url).await {
            slog::error!(log, "{e}");
            #[cfg(feature = "sentry")]
            sentry::capture_error(&e);
        }
    }
}
