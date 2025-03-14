use std::{convert::TryInto, process::Stdio};

use tokio::io::{AsyncBufReadExt, BufReader};

use super::{flag::Flag, output::Output, shell::Shell};
use crate::{bencher::sub::RunError, parser::project::run::CliRunShell};
use crate::{cli_eprintln_quietable, cli_println_quietable};

#[derive(Debug)]
pub enum Command {
    Shell {
        shell: Shell,
        flag: Flag,
        command: String,
    },
    Exec {
        program: String,
        arguments: Vec<String>,
    },
}

impl Command {
    pub fn new_shell(sh_c: CliRunShell, command: String) -> Result<Self, RunError> {
        let CliRunShell { shell, flag } = sh_c;
        Ok(Self::Shell {
            shell: shell.try_into()?,
            flag: flag.try_into()?,
            command,
        })
    }

    pub fn new_exec(program: String, arguments: Vec<String>) -> Self {
        Self::Exec { program, arguments }
    }

    pub async fn run(&self, log: bool) -> Result<Output, RunError> {
        let mut child = match self {
            Self::Shell {
                shell,
                flag,
                command,
            } => tokio::process::Command::new(shell.as_ref())
                .arg(flag.as_ref())
                .arg(command)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn(),
            Self::Exec { program, arguments } => tokio::process::Command::new(program)
                .args(arguments)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn(),
        }
        .map_err(RunError::SpawnCommand)?;

        let child_stdout = child.stdout.take().ok_or(RunError::PipeStdout)?;
        let stdout = tokio::spawn(async move {
            let stdout_reader = BufReader::new(child_stdout);
            let mut stdout_lines = stdout_reader.lines();

            let mut stdout = String::new();
            while let Ok(Some(line)) = stdout_lines.next_line().await {
                cli_println_quietable!(log, "{line}");
                if stdout.is_empty() {
                    stdout = line;
                } else {
                    stdout = format!("{stdout}\n{line}");
                }
            }

            stdout
        });

        let child_stderr = child.stderr.take().ok_or(RunError::PipeStderr)?;
        let stderr = tokio::spawn(async move {
            let stderr_reader = BufReader::new(child_stderr);
            let mut stderr_lines = stderr_reader.lines();

            let mut stderr = String::new();
            while let Ok(Some(line)) = stderr_lines.next_line().await {
                cli_eprintln_quietable!(log, "{line}");
                if stderr.is_empty() {
                    stderr = line;
                } else {
                    stderr = format!("{stderr}\n{line}");
                }
            }

            stderr
        });

        let (status, stdout, stderr) = tokio::join!(child.wait(), stdout, stderr);
        let status = status.map_err(RunError::RunCommand)?;
        let stdout = stdout.map_err(RunError::StdoutJoinError)?;
        let stderr = stderr.map_err(RunError::StderrJoinError)?;

        Ok(Output {
            status: status.into(),
            stdout,
            stderr,
            result: None,
        })
    }
}
