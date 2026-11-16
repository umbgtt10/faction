// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::process::command_outcome::CommandOutcome;
use crate::process::command_runner::CommandRunner;
use crate::process::launch_failure::LaunchFailure;
use std::env::consts::EXE_SUFFIX;
use std::env::split_paths;
use std::env::var_os;
use std::path::PathBuf;
use std::process::Command;

// The operating system's processes, each started with the environment this
// runner was given beside the one it inherits: the run's name for the system
// tests reaches every process slotgate starts that way, with no variable of
// this process's own set.
pub struct SystemCommandRunner {
    environment: Vec<(String, String)>,
}

impl SystemCommandRunner {
    pub fn new(environment: Vec<(String, String)>) -> Self {
        Self { environment }
    }

    fn command(&self, program: &str, args: &[String]) -> Command {
        let mut command = Command::new(program);
        command.args(args).envs(
            self.environment
                .iter()
                .map(|(name, value)| (name.as_str(), value.as_str())),
        );
        command
    }

    fn locate(binary_name: &str) -> Option<PathBuf> {
        let exe_name = format!("{binary_name}{EXE_SUFFIX}");
        let path_var = var_os("PATH")?;
        split_paths(&path_var)
            .map(|directory| directory.join(&exe_name))
            .find(|candidate| candidate.is_file())
    }
}

impl CommandRunner for SystemCommandRunner {
    fn is_available(&self, binary_name: &str) -> bool {
        Self::locate(binary_name).is_some()
    }

    fn run_streaming(&self, program: &str, args: &[String]) -> Result<Option<i32>, LaunchFailure> {
        self.command(program, args)
            .status()
            .map(|status| status.code())
            .map_err(|error| LaunchFailure::new(program, &error.to_string()))
    }

    fn run_capturing(
        &self,
        program: &str,
        args: &[String],
    ) -> Result<CommandOutcome, LaunchFailure> {
        let output = self
            .command(program, args)
            .output()
            .map_err(|error| LaunchFailure::new(program, &error.to_string()))?;

        Ok(CommandOutcome::new(
            output.status.code(),
            String::from_utf8_lossy(&output.stdout).into_owned(),
            String::from_utf8_lossy(&output.stderr).into_owned(),
        ))
    }
}
