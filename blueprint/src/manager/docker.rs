use docktopus::bollard::models::PortBinding;
use futures::TryFutureExt;
use std::collections::{BTreeMap, HashMap};
use tokio::process::Command;
use tokio_util::sync::CancellationToken;

use crate::error::Error;
use crate::manager::McpRunner;

/// Docker runner
#[derive(Debug, Clone)]
pub struct DockerRunner;

impl DockerRunner {
    /// Detect the current operating system
    async fn detect_os(&self) -> Result<String, Error> {
        let output = Command::new("uname")
            .arg("-s")
            .output()
            .await
            .map_err(Error::Io)?;

        if output.status.success() {
            let os = String::from_utf8_lossy(&output.stdout)
                .trim()
                .to_lowercase();
            Ok(os)
        } else {
            Err(Error::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Failed to detect operating system",
            )))
        }
    }

    /// Install Docker based on the detected operating system
    async fn install_docker(&self) -> Result<(), Error> {
        let os = self.detect_os().await?;
        blueprint_sdk::debug!(?os, "Detected operating system");

        match os.as_str() {
            "linux" => self.install_docker_linux().await,
            _ => Err(Error::Io(std::io::Error::new(
                std::io::ErrorKind::Unsupported,
                format!(
                    "Docker installation only supported on Linux, detected platform: {}",
                    os
                ),
            ))),
        }
    }

    /// Install Docker on Linux
    async fn install_docker_linux(&self) -> Result<(), Error> {
        blueprint_sdk::debug!("Installing Docker on Linux");

        // Use Docker's official installation script
        let status = Command::new("sh")
            .arg("-c")
            .arg("curl -fsSL https://get.docker.com -o get-docker.sh && sh get-docker.sh")
            .status()
            .await
            .map_err(Error::Io)?;

        if !status.success() {
            return Err(Error::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Docker installation script failed on Linux",
            )));
        }

        // Start Docker service if systemctl is available
        let _ = Command::new("sudo")
            .args(["systemctl", "start", "docker"])
            .status()
            .await;

        // Enable Docker service to start on boot
        let _ = Command::new("sudo")
            .args(["systemctl", "enable", "docker"])
            .status()
            .await;

        blueprint_sdk::debug!("Docker installed successfully on Linux");
        Ok(())
    }
}

impl McpRunner for DockerRunner {
    #[tracing::instrument(skip(self, ctx), fields(%package, args, port_bindings, runtime = "docker"))]
    async fn start(
        &self,
        ctx: &crate::MyContext,
        package: String,
        args: Vec<String>,
        port_bindings: Vec<(u16, Option<u16>)>,
        env_vars: BTreeMap<String, String>,
    ) -> Result<(CancellationToken, String), Error> {
        // Ensure Docker is available
        let mut checked = self.check(ctx).await;
        blueprint_sdk::debug!(?checked, "Checking if Docker is available");
        if !matches!(checked, Ok(true)) {
            // Try to install if not present or check errored
            blueprint_sdk::debug!("Installing Docker");
            self.install(ctx).await?;
            checked = self.check(ctx).await;
            if !matches!(checked, Ok(true)) {
                blueprint_sdk::debug!(?checked, "Docker install status");
                return Err(Error::Io(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "Docker is not available and could not be installed",
                )));
            }
        }

        // Determine endpoint from first host port
        let endpoint = port_bindings
            .first()
            .map(|(host_port, _)| format!("http://127.0.0.1:{}", host_port))
            .ok_or_else(|| Error::MissingPortBinding)?;

        // Use the struct's docker client
        let docker_client = ctx.docker.clone();

        // Since docktopus v0.3.0 doesn't support port bindings in Container API,
        // we need to create the container manually using bollard Config
        use docktopus::bollard::container::{
            Config, CreateContainerOptions, RemoveContainerOptions, StartContainerOptions,
            StopContainerOptions,
        };
        use docktopus::bollard::models::HostConfig;

        // Configure port bindings for Docker
        let mut port_bindings_map: HashMap<String, Option<Vec<PortBinding>>> = HashMap::new();
        for (host_port, container_port) in port_bindings {
            let container_port_str = container_port.unwrap_or(host_port).to_string();
            let port_binding = PortBinding {
                host_ip: Some("127.0.0.1".to_string()),
                host_port: Some(host_port.to_string()),
            };
            port_bindings_map.insert(
                format!("{}/tcp", container_port_str),
                Some(vec![port_binding]),
            );
        }

        // Convert environment variables to Vec<String> format
        let env: Vec<String> = env_vars
            .iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect();

        // Create container configuration with port bindings
        let config = Config {
            image: Some(package.clone()),
            cmd: Some(args),
            env: Some(env),
            attach_stdout: Some(true),
            host_config: Some(HostConfig {
                port_bindings: Some(port_bindings_map),
                ..Default::default()
            }),
            ..Default::default()
        };

        // Create the container directly using bollard
        let create_response = docker_client
            .create_container(None::<CreateContainerOptions<String>>, config)
            .await
            .map_err(|e| {
                Error::Io(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("Failed to create Docker container: {}", e),
                ))
            })?;

        let container_id = create_response.id;
        blueprint_sdk::debug!(?container_id, "Created Docker container");

        // Start the container
        docker_client
            .start_container(&container_id, None::<StartContainerOptions<String>>)
            .await
            .map_err(|e| {
                Error::Io(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("Failed to start Docker container: {}", e),
                ))
            })?;

        blueprint_sdk::debug!(?container_id, "Started Docker container");

        // Create cancellation token that will stop the container when cancelled
        let cancellation_token = CancellationToken::new();
        let stop_token = cancellation_token.clone();
        let stop_docker_client = docker_client.clone();

        tokio::spawn(async move {
            stop_token.cancelled().await;
            blueprint_sdk::debug!(?container_id, "Stopping Docker container");

            if let Err(e) = stop_docker_client
                .stop_container(&container_id, None::<StopContainerOptions>)
                .await
            {
                blueprint_sdk::error!(?e, ?container_id, "Failed to stop Docker container");
            }
            if let Err(e) = stop_docker_client
                .remove_container(&container_id, None::<RemoveContainerOptions>)
                .await
            {
                blueprint_sdk::error!(?e, ?container_id, "Failed to remove Docker container");
            }
        });

        Ok((cancellation_token, endpoint))
    }

    async fn check(&self, _ctx: &crate::MyContext) -> Result<bool, Error> {
        let status = Command::new("docker")
            .arg("--version")
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .map_err(Error::Io)
            .await?;
        Ok(status.success())
    }

    #[tracing::instrument(skip(self, _ctx), fields(runtime = "docker"))]
    async fn install(&self, _ctx: &crate::MyContext) -> Result<(), Error> {
        self.install_docker().await
    }
}
