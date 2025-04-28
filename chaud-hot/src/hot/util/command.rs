use anyhow::{Context as _, Result, ensure};
use std::process::{Command, Output};

/// Helper for properly checking [`Command`] results.
pub trait CommandExt {
    fn output_checked(&mut self) -> Result<Output>;

    fn stdout_str(&mut self) -> Result<String> {
        let output = self.output_checked()?;
        let stdout = String::from_utf8(output.stdout).context("Invalid stdout")?;
        Ok(stdout)
    }
}

impl CommandExt for Command {
    fn output_checked(&mut self) -> Result<Output> {
        let output = self.output().context("Command failed to run")?;
        ensure!(
            output.status.success(),
            "Command failed with status: {}",
            output.status
        );
        Ok(output)
    }
}
