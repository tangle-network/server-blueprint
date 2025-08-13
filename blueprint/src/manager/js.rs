use std::collections::BTreeMap;
use tokio::process::Command;
use tokio_util::sync::CancellationToken;

use crate::error::Error;
use crate::manager::ServerRunner;

/// JavaScript runner
///
/// This runner uses the `bun` package to run JavaScript scripts
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct JsRunner;

impl ServerRunner for JsRunner {
    #[tracing::instrument(skip(self, ctx), fields(%package, args, runtime = "js"))]
    async fn start(
        &self,
        ctx: &crate::MyContext,
        service_id: u64,
        package: String,
        args: Vec<String>,
        env_vars: BTreeMap<String, String>,
    ) -> Result<CancellationToken, Error> {
        // Ensure bun is installed
        let mut checked = self.check(ctx).await;
        blueprint_sdk::debug!(?checked, "Checking if bun is installed");
        if !matches!(checked, Ok(true)) {
            // Try to install if not present or check errored
            blueprint_sdk::debug!("Installing bun");
            self.install(ctx).await?;
            checked = self.check(ctx).await;
            if !matches!(checked, Ok(true)) {
                blueprint_sdk::debug!(?checked, "bun install status");
                return Err(Error::Io(std::io::Error::other(
                    "bun is not installed and could not be installed",
                )));
            }
        }

        blueprint_sdk::debug!("Starting JavaScript server with bunx");

        // Start the JavaScript process directly
        let mut cmd = Command::new("bunx");
        cmd.arg("-y")
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
                blueprint_sdk::debug!(?pid, "Terminating JavaScript process");
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
        let status = Command::new("bun")
            .arg("--version")
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .await
            .map_err(Error::Io)?;
        Ok(status.success())
    }

    #[tracing::instrument(skip(self, _ctx), fields(runtime = "js"))]
    async fn install(&self, _ctx: &crate::MyContext) -> Result<(), Error> {
        blueprint_sdk::debug!("Installing bun");
        let output = Command::new("sh")
            .arg("-c")
            .arg("curl -fsSL https://bun.sh/install | bash")
            .status()
            .await
            .map_err(Error::Io)?;
        if output.success() {
            blueprint_sdk::debug!("bun installed successfully");
            Ok(())
        } else {
            Err(Error::Io(std::io::Error::other(
                "bun installation script failed",
            )))
        }
    }
}
