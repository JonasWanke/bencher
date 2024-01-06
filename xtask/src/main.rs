#![allow(clippy::print_stdout, clippy::print_stderr, clippy::unused_self)]

mod parser;
mod task;

use task::Task;

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    dotenvy::from_path("xtask/.env").ok();
    exec().await
}

async fn exec() -> anyhow::Result<()> {
    let task = Task::new()?;
    task.exec().await
}
