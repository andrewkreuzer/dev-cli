#![allow(unused_imports)]

use std::{fs, path::Path, process::Command};

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use log::{debug, info};

#[cfg(feature = "python")]
use pyo3::prelude::*;
use pyo3::types::IntoPyDict;

use super::{Dev, RunStatus};

#[derive(Debug, Clone)]
pub struct PythonLanguage {}

impl PythonLanguage {
    pub fn new() -> Self {
        Self {}
    }

    #[cfg(feature = "python")]
    fn init(&self, dev: &Dev) -> Result<(), anyhow::Error> {
        pyo3::append_to_inittab!(dev);
        pyo3::prepare_freethreaded_python();

        Python::with_gil(|py| {
            let os = py.import_bound("os")?;
            let environ = os.getattr("environ")?;
            let env_vars = dev.environment.clone().into_py_dict_bound(py);
            environ.call_method1("update", (env_vars,))?;

            Ok(())
        })
    }
}

impl Default for PythonLanguage {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl super::LanguageFunctions for PythonLanguage {
    #[allow(unused_variables)]
    async fn run_file(
        &self,
        dev: Dev,
        file: &str,
        args: Vec<&str>,
    ) -> Result<RunStatus, anyhow::Error> {
        #[cfg(not(feature = "python"))]
        return Err(anyhow!("python support is not enabled"))?;

        #[cfg(feature = "python")]
        return self.run_file(dev, file, args).await;
    }

    #[allow(unused_variables)]
    async fn load_file(&self, file: &str) -> Result<(), anyhow::Error> {
        #[cfg(not(feature = "python"))]
        return Err(anyhow!("python support is not enabled"))?;

        #[cfg(feature = "python")]
        return self.load_file(file).await;
    }

    #[allow(unused_variables)]
    async fn run_shell(&self, command: &str, args: Vec<&str>) -> Result<RunStatus, anyhow::Error> {
        #[cfg(not(feature = "python"))]
        return Err(anyhow!("python support is not enabled"))?;

        #[cfg(feature = "python")]
        return self.run_shell(command, args).await;
    }
}

#[cfg(feature = "python")]
impl PythonLanguage {
    async fn run_file(
        &self,
        dev: Dev,
        file: &str,
        _args: Vec<&str>,
    ) -> Result<RunStatus, anyhow::Error> {
        self.init(&dev)?;

        Python::with_gil(|py| {
            let file_contents = fs::read_to_string(Path::new(file))?;
            let dev_out: Dev =
                PyModule::from_code_bound(py, &file_contents, "version", "version_info")?
                    .getattr("build")?
                    .extract()?;

            debug!(target: "python", "{:?}", dev_out);

            Ok(RunStatus {
                exit_code: Some(0),
                message: Some("success".to_string()),
            })
        })
    }

    async fn load_file(&self, _file: &str) -> Result<(), anyhow::Error> {
        todo!()
    }

    async fn run_shell(
        &self,
        _command: &str,
        _args: Vec<&str>,
    ) -> Result<RunStatus, anyhow::Error> {
        todo!();
    }
}

#[cfg(feature = "python")]
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
