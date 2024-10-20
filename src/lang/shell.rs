use std::process::Command;

use super::{Dev, RunError, RunStatus};

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

impl super::LanguageFunctions for ShellLanguage {
    async fn run_file(&self, _dev: Dev, file: &str, args: Vec<&str>) -> Result<RunStatus, anyhow::Error> {
        let cmd = format!("{} {}", file, args.join(" "));
        let mut child = Command::new(self.shell.as_str())
            .arg("-c")
            .arg(cmd)
            .spawn()
            .expect("failed to execute child");

        match child.wait()?.code() {
            Some(code) => {
                if code != 0 {
                    Err(anyhow::anyhow!(RunError {
                        exit_code: Some(code),
                        message: format!("Failed to run file: {file}, got {code}"),
                    }))
                } else {
                    Ok(RunStatus {
                        code,
                        message: None,
                    })
                }
            }
            None => Err(anyhow::anyhow!(RunError {
                exit_code: None,
                message: format!("Failed to run file: {file}, process terminated"),
            })),
        }
    }

    async fn load_file(&self, _file: &str) -> Result<(), anyhow::Error> {
        todo!()
    }

    async fn run_shell(&self, command: &str, args: Vec<&str>) -> Result<RunStatus, anyhow::Error> {
        // let command: Vec<&str> = command.split_whitespace().collect();

        let cmd = format!("{} {}", command, args.join(" "));
        let mut child = Command::new(self.shell.as_str())
            .arg("-c")
            .arg(cmd)
            .spawn()
            .expect("failed to execute child");

        match child.wait()?.code() {
            Some(code) => {
                if code != 0 {
                    Err(anyhow::anyhow!("Failed to run shell, exit code: {code}"))
                } else {
                    Ok(RunStatus {
                        code,
                        message: None,
                    })
                }
            }
            None => Err(anyhow::anyhow!("Failed to run shell, process terminated")),
        }
    }
}
