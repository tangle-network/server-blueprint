use std::collections::BTreeMap;
use std::process::{Child, Command, Stdio};

use crate::error::Error;
use crate::manager::McpRunner;

/// JavaScript runner
///
/// This runner uses the `bun` package to run JavaScript scripts
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct JsRunner;

impl McpRunner for JsRunner {
    fn start(
        &self,
        package: String,
        args: Vec<String>,
        port_bindings: Vec<(u16, Option<u16>)>,
        env_vars: BTreeMap<String, String>,
    ) -> Result<(Child, String), Error> {
        // Ensure bun is installed
        let mut checked = self.check();
        if !matches!(checked, Ok(true)) {
            // Try to install if not present or check errored
            self.install()?;
            checked = self.check();
            if !matches!(checked, Ok(true)) {
                return Err(Error::Io(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "bun is not installed and could not be installed",
                )));
            }
        }

        // Determine endpoint from first host port
        let endpoint = port_bindings
            .first()
            .map(|(host_port, _)| format!("http://127.0.0.1:{}", host_port))
            .ok_or_else(|| Error::MissingPortBinding)?;

        let mut cmd = Command::new("bunx");
        cmd.arg(&package).arg("--");
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
        let status = std::process::Command::new("bun")
            .arg("--version")
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .map_err(Error::Io)?;
        Ok(status.success())
    }

    fn install(&self) -> Result<(), Error> {
        let output = std::process::Command::new("sh")
            .arg("-c")
            .arg("curl -fsSL https://bun.sh/install | bash")
            .status()
            .map_err(Error::Io)?;
        if output.success() {
            Ok(())
        } else {
            Err(Error::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                "bun installation script failed",
            )))
        }
    }
}
