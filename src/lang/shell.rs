use std::process::Command;

use super::Dev;

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
    async fn run_file(&self, _dev: Dev, file: &str) -> Result<(), anyhow::Error> {
        let mut child = Command::new(self.shell.as_str())
            .arg("-c")
            .arg(file)
            .spawn()
            .expect("failed to execute child");

        match child.wait()?.code() {
            Some(code) => Err(anyhow::anyhow!("Failed to run file: {file}, got {code}")),
            None => Err(anyhow::anyhow!("Failed to run file: {file}")),
        }
    }

    async fn load_file(&self, _file: &str) -> Result<(), anyhow::Error> {
        todo!()
    }

    async fn run_shell(&self, command: &str) -> Result<(), anyhow::Error> {
        // let command: Vec<&str> = command.split_whitespace().collect();
        let mut child = Command::new(self.shell.as_str())
            .arg("-c")
            .arg(command)
            .spawn()
            .expect("failed to execute child");

        match child.wait()?.code() {
            Some(code) => Err(anyhow::anyhow!("Failed to run cmd: {command}, got {code}")),
            None => Err(anyhow::anyhow!("Failed to run cmd: {command}")),
        }
    }
}
