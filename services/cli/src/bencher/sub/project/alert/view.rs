use std::convert::TryFrom;

use async_trait::async_trait;
use bencher_json::{AlertUuid, JsonAlert, ResourceId};

use crate::{
    bencher::{backend::Backend, sub::SubCmd},
    parser::project::alert::CliAlertView,
    CliError,
};

#[derive(Debug)]
pub struct View {
    pub project: ResourceId,
    pub alert: AlertUuid,
    pub backend: Backend,
}

impl TryFrom<CliAlertView> for View {
    type Error = CliError;

    fn try_from(view: CliAlertView) -> Result<Self, Self::Error> {
        let CliAlertView {
            project,
            alert,
            backend,
        } = view;
        Ok(Self {
            project,
            alert,
            backend: backend.try_into()?,
        })
    }
}

#[async_trait]
impl SubCmd for View {
    async fn exec(&self) -> Result<(), CliError> {
        let _json: JsonAlert = self
            .backend
            .send_with(|client| async move {
                client
                    .proj_alert_get()
                    .project(self.project.clone())
                    .alert(self.alert)
                    .send()
                    .await
            })
            .await?;
        Ok(())
    }
}
