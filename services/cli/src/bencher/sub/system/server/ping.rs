use std::convert::TryFrom;

use async_trait::async_trait;
use bencher_json::JsonPing;

use crate::{
    bencher::{backend::Backend, sub::SubCmd},
    parser::system::server::CliPing,
    CliError,
};

#[derive(Debug, Clone)]
pub struct Ping {
    pub backend: Backend,
}

impl TryFrom<CliPing> for Ping {
    type Error = CliError;

    fn try_from(ping: CliPing) -> Result<Self, Self::Error> {
        let CliPing { backend } = ping;
        Ok(Self {
            backend: backend.try_into()?,
        })
    }
}

#[async_trait]
impl SubCmd for Ping {
    async fn exec(&self) -> Result<(), CliError> {
        let _json: JsonPing = self
            .backend
            .send_with(|client| async move { client.server_ping_get().send().await })
            .await?;
        Ok(())
    }
}
