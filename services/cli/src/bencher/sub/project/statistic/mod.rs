use crate::{bencher::sub::SubCmd, parser::project::statistic::CliStatistic, CliError};

mod view;

#[derive(Debug)]
pub enum Statistic {
    View(view::View),
}

impl TryFrom<CliStatistic> for Statistic {
    type Error = CliError;

    fn try_from(alert: CliStatistic) -> Result<Self, Self::Error> {
        Ok(match alert {
            CliStatistic::View(view) => Self::View(view.try_into()?),
        })
    }
}

impl SubCmd for Statistic {
    async fn exec(&self) -> Result<(), CliError> {
        match self {
            Self::View(view) => view.exec().await,
        }
    }
}
