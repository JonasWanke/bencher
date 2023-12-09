use std::convert::TryFrom;

use async_trait::async_trait;
use bencher_json::{JsonReport, ReportUuid, ResourceId};

use crate::{
    bencher::{backend::Backend, sub::SubCmd},
    parser::project::report::CliReportView,
    CliError,
};

#[derive(Debug)]
pub struct View {
    pub project: ResourceId,
    pub report: ReportUuid,
    pub backend: Backend,
}

impl TryFrom<CliReportView> for View {
    type Error = CliError;

    fn try_from(view: CliReportView) -> Result<Self, Self::Error> {
        let CliReportView {
            project,
            report,
            backend,
        } = view;
        Ok(Self {
            project,
            report,
            backend: backend.try_into()?,
        })
    }
}

#[async_trait]
impl SubCmd for View {
    async fn exec(&self) -> Result<(), CliError> {
        let _json: JsonReport = self
            .backend
            .send_with(|client| async move {
                client
                    .proj_report_get()
                    .project(self.project.clone())
                    .report(self.report)
                    .send()
                    .await
            })
            .await?;
        Ok(())
    }
}
