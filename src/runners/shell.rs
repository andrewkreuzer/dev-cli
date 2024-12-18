use std::process::Command;

use async_trait::async_trait;
use log::debug;

use super::{dev::Dev, language, RunError, RunStatus};

#[derive(Debug, Clone)]
pub struct ShellLanguage {
    shell: String,
}

impl ShellLanguage {
    pub fn new(shell: &str) -> Self {
        Self {
            shell: shell.to_string(),
        }
    }
}

impl Default for ShellLanguage {
    fn default() -> Self {
        Self::new("bash")
    }
}

#[async_trait]
impl language::LanguageFunctions for ShellLanguage {
    async fn run_file(
        &self,
        dev: Dev,
        file: &str,
        args: Vec<&str>,
    ) -> Result<RunStatus, anyhow::Error> {
        let cmd = format!("{} {}", file, args.join(" "));

        debug!(
            "running cmd: {} in shell: {} with envs: {}",
            cmd,
            self.shell,
            dev.get_env()
                .iter()
                .fold(
                    "".to_string(),
                    |acc, e| acc + &format!("{}={} ", e.0, e.1)
                )
        );

        let mut child = Command::new(self.shell.as_str())
            .arg("-c")
            .arg(cmd)
            .envs(dev.get_env().clone())
            .spawn()
            .expect("failed to execute child");

        match child.wait()?.code() {
            Some(code) => {
                if code != 0 {
                    Err(anyhow::anyhow!(RunError {
                        exit_code: Some(code),
                        message: Some(format!("Failed to run file: {file}, got {code}")),
                    }))
                } else {
                    Ok(RunStatus {
                        exit_code: Some(code),
                        message: None,
                    })
                }
            }
            None => Err(anyhow::anyhow!(RunError {
                exit_code: None,
                message: Some(format!("Failed to run file: {file}, process terminated")),
            })),
        }
    }

    async fn load_file(&self, _file: &str) -> Result<(), anyhow::Error> {
        todo!()
    }

    async fn run_shell(
        &self,
        _command: &str,
        _args: Vec<&str>,
    ) -> Result<RunStatus, anyhow::Error> {
        todo!()
    }
}
