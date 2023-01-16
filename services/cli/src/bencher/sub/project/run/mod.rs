use std::{convert::TryFrom, str::FromStr};

use async_trait::async_trait;
use bencher_json::{
    project::{
        branch::BRANCH_MAIN_STR, report::JsonReportSettings, testbed::TESTBED_LOCALHOST_STR,
    },
    BranchName, GitHash, JsonBranch, JsonNewReport, JsonReport, ResourceId,
};
use chrono::Utc;
use clap::ValueEnum;
use uuid::Uuid;

use crate::{
    bencher::locality::Locality,
    cli::project::run::{CliRun, CliRunAdapter},
    cli_println, CliError,
};

mod adapter;
mod fold;
mod runner;

use adapter::RunAdapter;
use fold::Fold;
pub use runner::Output;
use runner::Runner;

use crate::bencher::SubCmd;

const BENCHER_PROJECT: &str = "BENCHER_PROJECT";
const BENCHER_BRANCH: &str = "BENCHER_BRANCH";
const BENCHER_BRANCH_NAME: &str = "BENCHER_BRANCH_NAME";
const BENCHER_TESTBED: &str = "BENCHER_TESTBED";
const BENCHER_ADAPTER: &str = "BENCHER_ADAPTER";
const BENCHER_CMD: &str = "BENCHER_CMD";

#[derive(Debug)]
pub struct Run {
    project: ResourceId,
    locality: Locality,
    runner: Runner,
    branch: Branch,
    hash: Option<GitHash>,
    testbed: ResourceId,
    adapter: Option<RunAdapter>,
    iter: usize,
    fold: Option<Fold>,
    allow_failure: bool,
    err: bool,
}

#[derive(Debug, Clone)]
enum Branch {
    ResourceId(ResourceId),
    Name(BranchName),
}

impl TryFrom<CliRun> for Run {
    type Error = CliError;

    fn try_from(run: CliRun) -> Result<Self, Self::Error> {
        let CliRun {
            project,
            locality,
            command,
            branch,
            if_branch,
            hash,
            testbed,
            adapter,
            iter,
            fold,
            allow_failure,
            err,
        } = run;
        Ok(Self {
            project: unwrap_project(project)?,
            locality: locality.try_into()?,
            runner: command.try_into()?,
            branch: map_branch(branch, if_branch)?,
            hash: map_hash(hash)?,
            testbed: unwrap_testbed(testbed)?,
            adapter: map_adapter(adapter),
            iter: iter.unwrap_or(1),
            fold: fold.map(Into::into),
            allow_failure,
            err,
        })
    }
}

fn unwrap_project(project: Option<ResourceId>) -> Result<ResourceId, CliError> {
    Ok(if let Some(project) = project {
        project
    } else if let Ok(env_project) = std::env::var(BENCHER_PROJECT) {
        env_project.parse()?
    } else {
        return Err(CliError::ProjectNotFound);
    })
}

fn map_branch(
    branch: Option<ResourceId>,
    if_branch: Option<BranchName>,
) -> Result<Branch, CliError> {
    Ok(if let Some(branch) = branch {
        Branch::ResourceId(branch)
    } else if let Ok(env_branch) = std::env::var(BENCHER_BRANCH) {
        env_branch.as_str().parse().map(Branch::ResourceId)?
    } else if let Some(name) = if_branch {
        Branch::Name(name)
    } else if let Ok(env_name) = std::env::var(BENCHER_BRANCH_NAME) {
        env_name.parse().map(Branch::Name)?
    } else {
        BRANCH_MAIN_STR.parse().map(Branch::ResourceId)?
    })
}

fn map_hash(hash: Option<String>) -> Result<Option<GitHash>, CliError> {
    Ok(if let Some(hash) = hash {
        Some(GitHash::from_str(&hash)?)
    } else {
        None
    })
}

fn unwrap_testbed(testbed: Option<ResourceId>) -> Result<ResourceId, CliError> {
    Ok(if let Some(testbed) = testbed {
        testbed
    } else if let Ok(env_testbed) = std::env::var(BENCHER_TESTBED) {
        env_testbed.as_str().parse()?
    } else {
        TESTBED_LOCALHOST_STR.parse()?
    })
}

fn map_adapter(adapter: Option<CliRunAdapter>) -> Option<RunAdapter> {
    if let Some(adapter) = adapter {
        Some(adapter.into())
    } else if let Ok(env_adapter) = std::env::var(BENCHER_ADAPTER) {
        if let Ok(cli_adapter) = CliRunAdapter::from_str(&env_adapter, false) {
            Some(cli_adapter.into())
        } else {
            None
        }
    } else {
        None
    }
}

#[async_trait]
impl SubCmd for Run {
    async fn exec(&self) -> Result<(), CliError> {
        let branch = match &self.branch {
            Branch::ResourceId(resource_id) => resource_id.clone(),
            Branch::Name(name) => {
                if let Some(uuid) = if_branch(&self.project, name.as_ref(), &self.locality).await? {
                    uuid.into()
                } else {
                    return Ok(());
                }
            },
        };

        let start_time = Utc::now();

        let mut results = Vec::with_capacity(self.iter);
        for _ in 0..self.iter {
            let output = self.runner.run()?;
            results.push(output.result);
        }

        // TODO disable when quiet
        for result in &results {
            cli_println!("{result}");
        }

        let report = JsonNewReport {
            branch,
            hash: self.hash.clone(),
            testbed: self.testbed.clone(),
            start_time,
            end_time: Utc::now(),
            results,
            settings: Some(JsonReportSettings {
                adapter: self.adapter.map(Into::into),
                fold: self.fold.map(Into::into),
                allow_failure: Some(self.allow_failure),
            }),
        };

        // TODO disable when quiet
        cli_println!("{}", serde_json::to_string_pretty(&report)?);

        match &self.locality {
            Locality::Local => {},
            Locality::Backend(backend) => {
                let value = backend
                    .post(&format!("/v0/projects/{}/reports", self.project), &report)
                    .await?;
                if self.err {
                    let json_report: JsonReport = serde_json::from_value(value)?;
                    if !json_report.alerts.is_empty() {
                        return Err(CliError::Alerts);
                    }
                }
            },
        }

        Ok(())
    }
}

async fn if_branch(
    project: &ResourceId,
    name: &str,
    locality: &Locality,
) -> Result<Option<Uuid>, CliError> {
    if let Locality::Backend(backend) = &locality {
        let value = backend
            .get(&format!("/v0/projects/{project}/branches?name={name}"))
            .await?;
        let mut json_branches: Vec<JsonBranch> = serde_json::from_value(value)?;
        let branch_count = json_branches.len();
        if let Some(branch) = json_branches.pop() {
            return if branch_count == 1 {
                Ok(Some(branch.uuid))
            } else {
                Err(CliError::BranchName(
                    project.to_string(),
                    name.into(),
                    branch_count,
                ))
            };
        }
    }

    cli_println!("Failed to find branch with name \"{name}\" in project \"{project}\". Skipping benchmark run.");
    Ok(None)
}
