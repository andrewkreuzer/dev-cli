use super::Dev;
use anyhow::Result;
use log::info;
use pyo3::prelude::*;
use std::{fs, path::Path, process::Command};

#[derive(Debug, Clone)]
pub struct PythonLanguage {}

impl PythonLanguage {
    pub fn new() -> Self {
        Self {}
    }
    fn init(&self) -> Result<()> {
        pyo3::append_to_inittab!(dev);
        pyo3::prepare_freethreaded_python();
        Ok(())
    }
}

impl Default for PythonLanguage {
    fn default() -> Self {
        Self::new()
    }
}

impl super::LanguageFunctions for PythonLanguage {
    async fn run_file(&self, _dev: Dev, file: &str) -> Result<(), anyhow::Error> {
        self.init()?;

        Python::with_gil(|py| {
            let file_contents = fs::read_to_string(Path::new(file))?;
            let d: Dev = PyModule::from_code_bound(py, &file_contents, "version", "version_info")?
                .getattr("build")?
                .extract()?;
            info!(target: "python", "out: {:?}", d);
            Ok(())
        })
    }

    async fn load_file(&self, _file: &str) -> Result<(), anyhow::Error> {
        todo!()
    }

    async fn run_shell(&self, _command: &str) -> Result<(), anyhow::Error> {
        todo!();
    }
}

#[pymodule]
mod dev {
    use super::*;

    #[pyfunction]
    fn get_version() -> PyResult<String> {
        Ok("0.1.0".to_string())
    }

    #[pyfunction]
    fn get_env() -> PyResult<String> {
        let env = String::from_utf8_lossy(&Command::new("env").output()?.stdout).to_string();
        Ok(env)
    }

    #[pyfunction]
    fn get_work_dir() -> PyResult<String> {
        let pwd = String::from_utf8_lossy(&Command::new("pwd").output()?.stdout).to_string();
        Ok(pwd)
    }
}
