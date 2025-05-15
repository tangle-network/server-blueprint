use std::collections::BTreeMap;
use std::process::{Child, Command, Stdio};

use crate::error::Error;
use crate::manager::McpRunner;

/// Python runner
/// This runner uses the `uv` package to run Python scripts
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct PythonRunner;

impl McpRunner for PythonRunner {
    #[tracing::instrument(skip(self), fields(%package, args, port_bindings, runtime = "python"))]
    fn start(
        &self,
        package: String,
        args: Vec<String>,
        port_bindings: Vec<(u16, Option<u16>)>,
        env_vars: BTreeMap<String, String>,
    ) -> Result<(Child, String), Error> {
        // Ensure uv is installed
        let mut checked = self.check();
        blueprint_sdk::debug!(?checked, "Checking if uv is installed");
        if !matches!(checked, Ok(true)) {
            // Try to install if not present or check errored
            blueprint_sdk::debug!("Installing uv");
            self.install()?;
            checked = self.check();
            if !matches!(checked, Ok(true)) {
                blueprint_sdk::debug!(?checked, "uv install status");
                return Err(Error::Io(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "uv is not installed and could not be installed",
                )));
            }
        }

        // Determine endpoint from first host port
        let endpoint = port_bindings
            .first()
            .map(|(host_port, _)| format!("http://127.0.0.1:{}", host_port))
            .ok_or_else(|| Error::MissingPortBinding)?;

        let mut cmd = Command::new("uvx");
        cmd.arg("run").arg(&package).arg("--");
        for arg in &args {
            cmd.arg(arg);
        }
        cmd.stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        for (k, v) in env_vars.iter() {
            cmd.env(k, v);
        }
        let child = cmd.spawn().map_err(Error::from)?;
        Ok((child, endpoint))
    }

    fn check(&self) -> Result<bool, Error> {
        let status = std::process::Command::new("uv")
            .arg("--version")
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .map_err(Error::Io)?;
        Ok(status.success())
    }

    #[tracing::instrument(skip(self), fields(runtime = "python"))]
    fn install(&self) -> Result<(), Error> {
        // Install uv
        blueprint_sdk::debug!("Installing uv");
        let uv_install_status = std::process::Command::new("sh")
            .arg("-c")
            .arg("curl -LsSf https://astral.sh/uv/install.sh | sh")
            .status()
            .map_err(Error::Io)?;
        blueprint_sdk::debug!(?uv_install_status, "uv install status");
        if !uv_install_status.success() {
            return Err(Error::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                "uv installation script failed",
            )));
        }

        blueprint_sdk::debug!("uv installed successfully");
        // Install Python using uv
        let python_install_status = std::process::Command::new("uv")
            .arg("python")
            .arg("install")
            .status()
            .map_err(Error::Io)?;
        if python_install_status.success() {
            blueprint_sdk::debug!("Python installed successfully");
            Ok(())
        } else {
            Err(Error::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                "uv python install command failed",
            )))
        }
    }
}
