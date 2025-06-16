use docktopus::bollard::image::{CreateImageOptions, ListImagesOptions};
use docktopus::bollard::models::PortBinding;
use futures::{FutureExt, StreamExt, TryFutureExt};
use std::collections::{BTreeMap, HashMap};
use tokio::process::Command;
use tokio_util::sync::CancellationToken;

use crate::SupportedTransportAdapter;
use crate::error::Error;
use crate::manager::McpRunner;
use crate::transport::SseServer;

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
            Err(Error::Io(std::io::Error::other(
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
            return Err(Error::Io(std::io::Error::other(
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

    /// Check if a Docker image exists locally using the bollard API
    ///
    /// This method queries the Docker daemon to see if the specified image is already
    /// present in the local image registry. It uses the Docker API's list_images endpoint
    /// with a reference filter to efficiently check for the image without downloading it.
    ///
    /// # Arguments
    /// * `docker_client` - A reference to the bollard Docker client for API communication
    /// * `image` - The Docker image name/tag to check (e.g., "nginx:latest", "ubuntu:20.04")
    ///
    /// # Returns
    /// * `Ok(true)` if the image exists locally
    /// * `Ok(false)` if the image does not exist locally
    /// * `Err(Error)` if there was an error communicating with the Docker daemon
    ///
    /// # Examples
    /// ```rust
    /// let exists = runner.check_image_exists(&docker_client, "nginx:latest").await?;
    /// if exists {
    ///     println!("Image is already available locally");
    /// }
    /// ```
    async fn check_image_exists(
        &self,
        docker_client: &docktopus::bollard::Docker,
        image: &str,
    ) -> Result<bool, Error> {
        // Configure the list images request to filter by the specific image reference
        // The 'reference' filter matches images by their name and tag
        let options = ListImagesOptions::<String> {
            all: false, // Only show non-intermediate images (not build cache layers)
            filters: HashMap::from([("reference".to_string(), vec![image.to_string()])]),
            ..Default::default()
        };

        // Query the Docker daemon for images matching our filter
        let images = docker_client
            .list_images(Some(options))
            .await
            .map_err(|e| {
                Error::Io(std::io::Error::other(format!(
                    "Failed to list Docker images: {}",
                    e
                )))
            })?;

        // If any images were returned, the image exists locally
        Ok(!images.is_empty())
    }

    /// Pull a Docker image from a registry using the bollard API
    ///
    /// This method downloads a Docker image from a registry (Docker Hub by default)
    /// to the local Docker daemon. It uses Docker's create_image API which streams
    /// the download progress and handles layers efficiently.
    ///
    /// # Arguments
    /// * `docker_client` - A reference to the bollard Docker client for API communication
    /// * `image` - The Docker image name/tag to pull (e.g., "nginx:latest", "ubuntu:20.04")
    ///
    /// # Returns
    /// * `Ok(())` if the image was successfully pulled
    /// * `Err(Error)` if there was an error during the pull operation
    ///
    /// # Behavior
    /// - Streams the download progress and logs status updates
    /// - Handles Docker registry authentication if configured
    /// - Automatically retries failed layer downloads (handled by Docker daemon)
    /// - Validates image integrity during download
    ///
    /// # Examples
    /// ```rust
    /// runner.pull_image(&docker_client, "nginx:latest").await?;
    /// println!("Image pulled successfully");
    /// ```
    async fn pull_image(
        &self,
        docker_client: &docktopus::bollard::Docker,
        image: &str,
    ) -> Result<(), Error> {
        blueprint_sdk::debug!(?image, "Pulling Docker image");

        use futures::StreamExt;

        // Configure the image pull request
        // from_image specifies the image name and tag to pull
        let options = CreateImageOptions {
            from_image: image,
            ..Default::default()
        };

        // Create a stream for the image pull operation
        // The Docker API returns progress updates as a stream of events
        let mut stream = docker_client.create_image(Some(options), None, None);

        // Process each event in the pull stream
        while let Some(result) = stream.next().await {
            match result {
                Ok(info) => {
                    // Check if the pull operation encountered an error
                    if let Some(error) = info.error {
                        return Err(Error::Io(std::io::Error::other(format!(
                            "Failed to pull Docker image {}: {}",
                            image, error
                        ))));
                    }
                    // Log progress updates for debugging and monitoring
                    if let Some(status) = info.status {
                        blueprint_sdk::debug!(?image, status, "Image pull progress");
                    }
                }
                Err(e) => {
                    // Handle stream errors (network issues, Docker daemon problems, etc.)
                    return Err(Error::Io(std::io::Error::other(format!(
                        "Failed to pull Docker image {}: {}",
                        image, e
                    ))));
                }
            }
        }

        blueprint_sdk::debug!(?image, "Docker image pulled successfully");
        Ok(())
    }

    /// Ensure a Docker image is available locally, pulling it if necessary
    ///
    /// This is a convenience method that combines image existence checking and pulling.
    /// It first checks if the image exists locally, and only pulls it if it's not found.
    /// This approach is efficient and avoids unnecessary network operations.
    ///
    /// # Arguments
    /// * `docker_client` - A reference to the bollard Docker client for API communication
    /// * `image` - The Docker image name/tag to ensure is available (e.g., "nginx:latest")
    ///
    /// # Returns
    /// * `Ok(())` if the image is available (either was already present or successfully pulled)
    /// * `Err(Error)` if there was an error checking for or pulling the image
    ///
    /// # Workflow
    /// 1. Check if the image exists locally using `check_image_exists()`
    /// 2. If image is not found, pull it using `pull_image()`
    /// 3. If image already exists, skip the pull operation
    ///
    /// # Examples
    /// ```rust
    /// // This will only pull if the image isn't already present
    /// runner.ensure_image_available(&docker_client, "nginx:latest").await?;
    /// println!("Image is now available for use");
    /// ```
    async fn ensure_image_available(
        &self,
        docker_client: &docktopus::bollard::Docker,
        image: &str,
    ) -> Result<(), Error> {
        // First check if the image is already available locally
        if !self.check_image_exists(docker_client, image).await? {
            // Image not found locally, need to pull it from registry
            blueprint_sdk::debug!(?image, "Image not found locally, pulling");
            self.pull_image(docker_client, image).await?;
        } else {
            // Image already exists, no action needed
            blueprint_sdk::debug!(?image, "Image already exists locally");
        }
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
        transport_adapter: SupportedTransportAdapter,
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
                return Err(Error::Io(std::io::Error::other(
                    "Docker is not available and could not be installed",
                )));
            }
        }

        // Determine endpoint from first host port
        let endpoint = port_bindings
            .first()
            .map(|(host_port, _)| format!("127.0.0.1:{}", host_port))
            .ok_or_else(|| Error::MissingPortBinding)?;

        // Use the struct's docker client
        let docker_client = ctx.docker.clone();

        // Ensure the Docker image is available locally (pull if not present)
        self.ensure_image_available(&docker_client, &package)
            .await?;

        // Since docktopus v0.3.0 doesn't support port bindings in Container API,
        // we need to create the container manually using bollard Config
        use docktopus::bollard::container::{
            AttachContainerOptions, Config, CreateContainerOptions, RemoveContainerOptions,
            StartContainerOptions, StopContainerOptions,
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
                Error::Io(std::io::Error::other(format!(
                    "Failed to create Docker container: {}",
                    e
                )))
            })?;

        let container_id = create_response.id;
        blueprint_sdk::debug!(?container_id, "Created Docker container");

        // Start the container
        docker_client
            .start_container(&container_id, None::<StartContainerOptions<String>>)
            .await
            .map_err(|e| {
                Error::Io(std::io::Error::other(format!(
                    "Failed to start Docker container: {}",
                    e
                )))
            })?;

        blueprint_sdk::debug!(?container_id, "Started Docker container");

        // Clone the necessary values for the factory closure
        let docker_client_factory = docker_client.clone();
        let factory_container_id = container_id.clone();

        // Create Docker transport factory using async block approach
        let factory = move || {
            let docker_client_clone = docker_client_factory.clone();
            let container_id_clone = factory_container_id.clone();

            if transport_adapter.is_none() {
                return futures::future::pending().boxed();
            }

            async move {
                // Attach to the container to get stdin/stdout streams
                let attach_options = AttachContainerOptions::<String> {
                    stdout: Some(true),
                    stdin: Some(true),
                    stream: Some(true),
                    ..Default::default()
                };

                match docker_client_clone
                    .attach_container(&container_id_clone, Some(attach_options))
                    .await
                {
                    Ok(res) => Ok(DockerTransport::new(res)),
                    Err(e) => Err(std::io::Error::other(format!(
                        "Failed to attach to Docker container: {e}"
                    ))),
                }
            }
            .boxed()
        };

        let ct = match transport_adapter {
            SupportedTransportAdapter::StdioToSSE => {
                SseServer::serve(endpoint.parse()?).await?.forward(factory)
            }
            SupportedTransportAdapter::None => CancellationToken::new(),
        };

        // Create cleanup task that will stop the container when cancelled
        let stop_docker_client = docker_client.clone();
        let cleanup_container_id = container_id.clone();
        let cleanup_ct = ct.clone();

        tokio::spawn(async move {
            cleanup_ct.cancelled().await;
            blueprint_sdk::debug!(?cleanup_container_id, "Stopping Docker container");

            if let Err(e) = stop_docker_client
                .stop_container(&cleanup_container_id, None::<StopContainerOptions>)
                .await
            {
                blueprint_sdk::error!(?e, ?cleanup_container_id, "Failed to stop Docker container");
            }
            if let Err(e) = stop_docker_client
                .remove_container(&cleanup_container_id, None::<RemoveContainerOptions>)
                .await
            {
                blueprint_sdk::error!(
                    ?e,
                    ?cleanup_container_id,
                    "Failed to remove Docker container"
                );
            }
        });

        Ok((ct, endpoint))
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

struct DockerTransport {
    results: docktopus::bollard::container::AttachContainerResults,
}

impl DockerTransport {
    fn new(results: docktopus::bollard::container::AttachContainerResults) -> Self {
        Self { results }
    }
}

impl<R, A> rmcp::transport::IntoTransport<R, std::io::Error, A> for DockerTransport
where
    R: rmcp::service::ServiceRole,
{
    fn into_transport(
        self,
    ) -> (
        impl futures::Sink<rmcp::service::TxJsonRpcMessage<R>, Error = std::io::Error> + Send + 'static,
        impl futures::Stream<Item = rmcp::service::RxJsonRpcMessage<R>> + Send + 'static,
    ) {
        let stream = self.results.output;
        let sink = rmcp::transport::io::from_async_write(self.results.input);
        let mapped_stream = stream.map(|msg| match msg {
            Ok(log) => serde_json::from_slice::<rmcp::service::RxJsonRpcMessage<R>>(log.as_ref())
                .map_err(|e| {
                    rmcp::service::RxJsonRpcMessage::<R>::Error(rmcp::model::JsonRpcError {
                        jsonrpc: rmcp::model::JsonRpcVersion2_0,
                        id: rmcp::model::NumberOrString::Number(1),
                        error: rmcp::model::ErrorData {
                            code: rmcp::model::ErrorCode::PARSE_ERROR,
                            message: e.to_string().into(),
                            data: None,
                        },
                    })
                })
                .unwrap(),
            Err(e) => rmcp::service::RxJsonRpcMessage::<R>::Error(rmcp::model::JsonRpcError {
                jsonrpc: rmcp::model::JsonRpcVersion2_0,
                id: rmcp::model::NumberOrString::Number(1),
                error: rmcp::model::ErrorData {
                    code: rmcp::model::ErrorCode::INTERNAL_ERROR,
                    message: e.to_string().into(),
                    data: None,
                },
            }),
        });

        (sink, mapped_stream)
    }
}
