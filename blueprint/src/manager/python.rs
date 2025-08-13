use std::collections::BTreeMap;

use tokio::process::Command;
use tokio_util::sync::CancellationToken;

use crate::error::Error;
use crate::manager::ServerRunner;

/// Python runner
/// This runner uses the `uv` package to run Python scripts
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct PythonRunner;

impl ServerRunner for PythonRunner {
    #[tracing::instrument(skip(self, ctx), fields(%package, args, runtime = "python"))]
    async fn start(
        &self,
        ctx: &crate::MyContext,
        service_id: u64,
        package: String,
        args: Vec<String>,
        env_vars: BTreeMap<String, String>,
    ) -> Result<CancellationToken, Error> {
        // Ensure uv is installed
        let mut checked = self.check(ctx).await;
        blueprint_sdk::debug!(?checked, "Checking if uv is installed");
        if !matches!(checked, Ok(true)) {
            // Try to install if not present or check errored
            blueprint_sdk::debug!("Installing uv");
            self.install(ctx).await?;
            checked = self.check(ctx).await;
            if !matches!(checked, Ok(true)) {
                blueprint_sdk::debug!(?checked, "uv install status");
                return Err(Error::Io(std::io::Error::other(
                    "uv is not installed and could not be installed",
                )));
            }
        }

        blueprint_sdk::debug!("Starting Python server with uvx");

        // Start the Python process directly
        let mut cmd = Command::new("uvx");
        cmd.arg("run")
            .arg(&package)
            .arg("--")
            .args(&args)
            .envs(&env_vars)
            .kill_on_drop(true);

        let child = cmd.spawn().map_err(Error::Io)?;
        
        // Create cancellation token for cleanup
        let ct = CancellationToken::new();
        let cleanup_ct = ct.clone();
        let child_id = child.id();

        // Spawn cleanup task
        tokio::spawn(async move {
            cleanup_ct.cancelled().await;
            if let Some(pid) = child_id {
                blueprint_sdk::debug!(?pid, "Terminating Python process");
                #[cfg(unix)]
                {
                    let _ = tokio::process::Command::new("kill")
                        .arg(pid.to_string())
                        .status()
                        .await;
                }
            }
        });

        Ok(ct)
    }

    async fn check(&self, _ctx: &crate::MyContext) -> Result<bool, Error> {
        let status = tokio::process::Command::new("uv")
            .arg("--version")
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .await
            .map_err(Error::Io)?;
        Ok(status.success())
    }

    #[tracing::instrument(skip(self, _ctx), fields(runtime = "python"))]
    async fn install(&self, _ctx: &crate::MyContext) -> Result<(), Error> {
        // Install uv
        blueprint_sdk::debug!("Installing uv");
        let uv_install_status = tokio::process::Command::new("sh")
            .arg("-c")
            .arg("curl -LsSf https://astral.sh/uv/install.sh | sh")
            .status()
            .await
            .map_err(Error::Io)?;
        blueprint_sdk::debug!(?uv_install_status, "uv install status");
        if !uv_install_status.success() {
            return Err(Error::Io(std::io::Error::other(
                "uv installation script failed",
            )));
        }

        blueprint_sdk::debug!("uv installed successfully");
        // Install Python using uv
        let python_install_status = tokio::process::Command::new("uv")
            .arg("python")
            .arg("install")
            .status()
            .await
            .map_err(Error::Io)?;
        if python_install_status.success() {
            blueprint_sdk::debug!("Python installed successfully");
            Ok(())
        } else {
            Err(Error::Io(std::io::Error::other(
                "uv python install command failed",
            )))
        }
    }
}
